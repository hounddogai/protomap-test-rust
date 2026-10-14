use super::*;

use acme_kit::text;

/// Where a query slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuerySlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QuerySlotStatus {
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

impl std::fmt::Display for QuerySlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuerySlot {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub starts_at: i64,
    pub timezone: String,
    pub scope: String,
    pub audience: String,
    pub source: String,
    pub quantity: i64,
    pub expires_at: i64,
    pub status: QuerySlotStatus,
}

impl QuerySlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuerySlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuerySlotStatus) -> bool {
        match self.status {
            QuerySlotStatus::Planned => {
                return next == QuerySlotStatus::InProgress;
            }
            QuerySlotStatus::Scheduled => {
                match next {
                    QuerySlotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuerySlotStatus::InProgress => {
                return next == QuerySlotStatus::Completed || next == QuerySlotStatus::Cancelled;
            }
            QuerySlotStatus::Completed => {
                return next == QuerySlotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuerySlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuerySlot {
        QuerySlot {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            starts_at: self.starts_at,
            timezone: self.timezone.clone(),
            scope: self.scope.clone(),
            audience: self.audience.clone(),
            source: self.source.clone(),
            quantity: self.quantity,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a query slot in status planned.
pub fn new_query_slot(code: &str, title: &str) -> QuerySlot {
    QuerySlot {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        starts_at: 0,
        timezone: "".to_string(),
        scope: "".to_string(),
        audience: "".to_string(),
        source: "".to_string(),
        quantity: 0,
        expires_at: 0,
        status: QuerySlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_query_slot("code-579", "title-140");
        assert_eq!(item.status, QuerySlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_query_slot("code-579", "title-140");
        if item.move_to(QuerySlotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuerySlotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_query_slot("code-579", "title-140");
        item.status = QuerySlotStatus::Cancelled;
        assert!(item.move_to(QuerySlotStatus::Planned).is_err(), "expected move to fail");
    }
}
