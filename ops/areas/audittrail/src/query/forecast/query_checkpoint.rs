use super::*;

use acme_kit::text;

/// Where a query checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QueryCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QueryCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryCheckpoint {
    pub code: String,
    pub name: String,
    pub title: String,
    pub summary: String,
    pub segment: String,
    pub phone: String,
    pub duration_minutes: i64,
    pub principal: String,
    pub maximum: f64,
    pub status: QueryCheckpointStatus,
}

impl QueryCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryCheckpointStatus) -> bool {
        match self.status {
            QueryCheckpointStatus::Proposed => {
                return next == QueryCheckpointStatus::Approved;
            }
            QueryCheckpointStatus::Approved => {
                return next == QueryCheckpointStatus::Withdrawn;
            }
            QueryCheckpointStatus::Rejected => {
                return next == QueryCheckpointStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query checkpoint", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.duration_minutes as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryCheckpoint {
        QueryCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            segment: self.segment.clone(),
            phone: self.phone.clone(),
            duration_minutes: self.duration_minutes,
            principal: self.principal.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a query checkpoint in status proposed.
pub fn new_query_checkpoint(code: &str, name: &str) -> QueryCheckpoint {
    QueryCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        summary: "".to_string(),
        segment: "".to_string(),
        phone: "".to_string(),
        duration_minutes: 0,
        principal: "".to_string(),
        maximum: 0.0,
        status: QueryCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_query_checkpoint("code-855", "name-865");
        assert_eq!(item.status, QueryCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_query_checkpoint("code-855", "name-865");
        if item.move_to(QueryCheckpointStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryCheckpointStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_query_checkpoint("code-855", "name-865");
        item.status = QueryCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(QueryCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
