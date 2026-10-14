use super::*;

use acme_kit::text;

/// Where a query score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl QueryScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for QueryScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query score as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryScore {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub currency: String,
    pub budget: f64,
    pub status: QueryScoreStatus,
}

impl QueryScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryScoreStatus) -> bool {
        match self.status {
            QueryScoreStatus::Created => {
                match next {
                    QueryScoreStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryScoreStatus::Validated => {
                return next == QueryScoreStatus::Retired;
            }
            QueryScoreStatus::Published => {
                match next {
                    QueryScoreStatus::Retired => {
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

    pub fn move_to(&mut self, next: QueryScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        (self.budget * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryScore {
        QueryScore {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            currency: self.currency.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a query score in status created.
pub fn new_query_score(code: &str, label: &str) -> QueryScore {
    QueryScore {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        currency: "".to_string(),
        budget: 0.0,
        status: QueryScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_query_score("code-969", "label-970");
        assert_eq!(item.status, QueryScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_query_score("code-969", "label-970");
        if item.move_to(QueryScoreStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryScoreStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_query_score("code-969", "label-970");
        item.status = QueryScoreStatus::Retired;
        assert!(
            item.move_to(QueryScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
