use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a query budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QueryBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for QueryBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryBudget {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub attributes: BTreeMap<String, String>,
    pub weight: f64,
    pub amount: f64,
    pub ends_at: i64,
    pub quantity: i64,
    pub source: String,
    pub updated_at: i64,
    pub status: QueryBudgetStatus,
}

impl QueryBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryBudgetStatus) -> bool {
        match self.status {
            QueryBudgetStatus::Planned => {
                match next {
                    QueryBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryBudgetStatus::Scheduled => {
                match next {
                    QueryBudgetStatus::InProgress | QueryBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryBudgetStatus::InProgress => {
                match next {
                    QueryBudgetStatus::Completed | QueryBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryBudgetStatus::Completed => {
                match next {
                    QueryBudgetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: QueryBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_value(&self) -> f64 {
        ((self.batch_size as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryBudget {
        QueryBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            attributes: self.attributes.clone(),
            weight: self.weight,
            amount: self.amount,
            ends_at: self.ends_at,
            quantity: self.quantity,
            source: self.source.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a query budget in status planned.
pub fn new_query_budget(code: &str, title: &str) -> QueryBudget {
    QueryBudget {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        attributes: BTreeMap::new(),
        weight: 0.0,
        amount: 0.0,
        ends_at: 0,
        quantity: 0,
        source: "".to_string(),
        updated_at: 0,
        status: QueryBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_query_budget("code-979", "title-738");
        assert_eq!(item.status, QueryBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_query_budget("code-979", "title-738");
        if item.move_to(QueryBudgetStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryBudgetStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_query_budget("code-979", "title-738");
        item.status = QueryBudgetStatus::Cancelled;
        assert!(
            item.move_to(QueryBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
