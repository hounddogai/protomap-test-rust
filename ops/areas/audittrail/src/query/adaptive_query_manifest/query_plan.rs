use super::*;

use acme_kit::text;

/// Where a query plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QueryPlanStatus {
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

impl std::fmt::Display for QueryPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlan {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub principal: String,
    pub label: String,
    pub revision: i64,
    pub rate: f64,
    pub timezone: String,
    pub status: QueryPlanStatus,
}

impl QueryPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryPlanStatus) -> bool {
        match self.status {
            QueryPlanStatus::Draft => {
                match next {
                    QueryPlanStatus::Active | QueryPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryPlanStatus::Pending => {
                match next {
                    QueryPlanStatus::Active | QueryPlanStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryPlanStatus::Active => {
                match next {
                    QueryPlanStatus::Suspended | QueryPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryPlanStatus::Suspended => {
                return next == QueryPlanStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.revision as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryPlan {
        QueryPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            principal: self.principal.clone(),
            label: self.label.clone(),
            revision: self.revision,
            rate: self.rate,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query plan in status draft.
pub fn new_query_plan(code: &str, title: &str) -> QueryPlan {
    QueryPlan {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        principal: "".to_string(),
        label: "".to_string(),
        revision: 0,
        rate: 0.0,
        timezone: "".to_string(),
        status: QueryPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_query_plan("code-981", "title-220");
        assert_eq!(item.status, QueryPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_query_plan("code-981", "title-220");
        if item.move_to(QueryPlanStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryPlanStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_query_plan("code-981", "title-220");
        item.status = QueryPlanStatus::Closed;
        assert!(item.move_to(QueryPlanStatus::Draft).is_err(), "expected move to fail");
    }
}
