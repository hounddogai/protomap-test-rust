use super::*;

use acme_kit::text;

/// Where a batched rollback target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRollbackTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedRollbackTargetStatus {
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

impl std::fmt::Display for BatchedRollbackTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched rollback target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackTarget {
    pub code: String,
    pub title: String,
    pub category: String,
    pub reviewer: String,
    pub source: String,
    pub visible: bool,
    pub attempts: i64,
    pub timezone: String,
    pub minimum: f64,
    pub status: BatchedRollbackTargetStatus,
}

impl BatchedRollbackTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRollbackTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRollbackTargetStatus) -> bool {
        match self.status {
            BatchedRollbackTargetStatus::Planned => {
                match next {
                    BatchedRollbackTargetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRollbackTargetStatus::Scheduled => {
                match next {
                    BatchedRollbackTargetStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRollbackTargetStatus::InProgress => {
                match next {
                    BatchedRollbackTargetStatus::Completed | BatchedRollbackTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRollbackTargetStatus::Completed => {
                return next == BatchedRollbackTargetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRollbackTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched rollback target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.attempts as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRollbackTarget {
        BatchedRollbackTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            visible: self.visible,
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched rollback target in status planned.
pub fn new_batched_rollback_target(code: &str, title: &str) -> BatchedRollbackTarget {
    BatchedRollbackTarget {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        reviewer: "".to_string(),
        source: "".to_string(),
        visible: false,
        attempts: 0,
        timezone: "".to_string(),
        minimum: 0.0,
        status: BatchedRollbackTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_rollback_target("code-460", "title-774");
        assert_eq!(item.status, BatchedRollbackTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_batched_rollback_target("code-460", "title-774");
        if item.move_to(BatchedRollbackTargetStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRollbackTargetStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_rollback_target("code-460", "title-774");
        item.status = BatchedRollbackTargetStatus::Cancelled;
        assert!(
            item.move_to(BatchedRollbackTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
