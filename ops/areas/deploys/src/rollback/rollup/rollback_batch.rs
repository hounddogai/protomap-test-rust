use super::*;

use acme_kit::text;

/// Where a rollback batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollbackBatchStatus {
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

impl std::fmt::Display for RollbackBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBatch {
    pub code: String,
    pub title: String,
    pub category: String,
    pub label: String,
    pub owner_id: String,
    pub score: f64,
    pub status: RollbackBatchStatus,
}

impl RollbackBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackBatchStatus) -> bool {
        match self.status {
            RollbackBatchStatus::Open => {
                match next {
                    RollbackBatchStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackBatchStatus::InReview => {
                return next == RollbackBatchStatus::Resolved;
            }
            RollbackBatchStatus::Resolved => {
                return next == RollbackBatchStatus::Reopened || next == RollbackBatchStatus::Archived;
            }
            RollbackBatchStatus::Reopened => {
                match next {
                    RollbackBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollbackBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> RollbackBatch {
        RollbackBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rollback batch in status open.
pub fn new_rollback_batch(code: &str, title: &str) -> RollbackBatch {
    RollbackBatch {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        label: "".to_string(),
        owner_id: "".to_string(),
        score: 0.0,
        status: RollbackBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollback_batch("code-208", "title-806");
        assert_eq!(item.status, RollbackBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_rollback_batch("code-208", "title-806");
        if item.move_to(RollbackBatchStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackBatchStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollback_batch("code-208", "title-806");
        item.status = RollbackBatchStatus::Archived;
        assert!(
            item.move_to(RollbackBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
