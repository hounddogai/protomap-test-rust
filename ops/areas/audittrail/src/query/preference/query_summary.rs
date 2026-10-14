use super::*;

use acme_kit::text;

/// Where a query summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuerySummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QuerySummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for QuerySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuerySummary {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub reviewer: String,
    pub namespace: String,
    pub score: f64,
    pub rate: f64,
    pub cluster: String,
    pub limit: i64,
    pub status: QuerySummaryStatus,
}

impl QuerySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuerySummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuerySummaryStatus) -> bool {
        match self.status {
            QuerySummaryStatus::Draft => {
                match next {
                    QuerySummaryStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuerySummaryStatus::Pending => {
                match next {
                    QuerySummaryStatus::Active | QuerySummaryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuerySummaryStatus::Active => {
                return next == QuerySummaryStatus::Suspended || next == QuerySummaryStatus::Closed;
            }
            QuerySummaryStatus::Suspended => {
                match next {
                    QuerySummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuerySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.score + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuerySummary {
        QuerySummary {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            reviewer: self.reviewer.clone(),
            namespace: self.namespace.clone(),
            score: self.score,
            rate: self.rate,
            cluster: self.cluster.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a query summary in status draft.
pub fn new_query_summary(code: &str, name: &str) -> QuerySummary {
    QuerySummary {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        reviewer: "".to_string(),
        namespace: "".to_string(),
        score: 0.0,
        rate: 0.0,
        cluster: "".to_string(),
        limit: 0,
        status: QuerySummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_query_summary("code-652", "name-328");
        assert_eq!(item.status, QuerySummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_query_summary("code-652", "name-328");
        if item.move_to(QuerySummaryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuerySummaryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_query_summary("code-652", "name-328");
        item.status = QuerySummaryStatus::Closed;
        assert!(
            item.move_to(QuerySummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
