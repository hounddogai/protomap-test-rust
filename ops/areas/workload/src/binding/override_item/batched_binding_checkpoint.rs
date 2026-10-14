use super::*;

use acme_kit::text;

/// Where a batched binding checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedBindingCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedBindingCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for BatchedBindingCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched binding checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingCheckpoint {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub ends_at: i64,
    pub category: String,
    pub country: String,
    pub source: String,
    pub margin: f64,
    pub notes: String,
    pub budget: f64,
    pub status: BatchedBindingCheckpointStatus,
}

impl BatchedBindingCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedBindingCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedBindingCheckpointStatus) -> bool {
        match self.status {
            BatchedBindingCheckpointStatus::Open => {
                match next {
                    BatchedBindingCheckpointStatus::InReview | BatchedBindingCheckpointStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedBindingCheckpointStatus::InReview => {
                match next {
                    BatchedBindingCheckpointStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedBindingCheckpointStatus::Resolved => {
                return next == BatchedBindingCheckpointStatus::Reopened
                    || next == BatchedBindingCheckpointStatus::Archived;
            }
            BatchedBindingCheckpointStatus::Reopened => {
                return next == BatchedBindingCheckpointStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedBindingCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched binding checkpoint {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.margin - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedBindingCheckpoint {
        BatchedBindingCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            ends_at: self.ends_at,
            category: self.category.clone(),
            country: self.country.clone(),
            source: self.source.clone(),
            margin: self.margin,
            notes: self.notes.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched binding checkpoint in status open.
pub fn new_batched_binding_checkpoint(code: &str, name: &str) -> BatchedBindingCheckpoint {
    BatchedBindingCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        ends_at: 0,
        category: "".to_string(),
        country: "".to_string(),
        source: "".to_string(),
        margin: 0.0,
        notes: "".to_string(),
        budget: 0.0,
        status: BatchedBindingCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_binding_checkpoint("code-840", "name-671");
        assert_eq!(item.status, BatchedBindingCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_batched_binding_checkpoint("code-840", "name-671");
        if item.move_to(BatchedBindingCheckpointStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedBindingCheckpointStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_binding_checkpoint("code-840", "name-671");
        item.status = BatchedBindingCheckpointStatus::Archived;
        assert!(
            item.move_to(BatchedBindingCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
