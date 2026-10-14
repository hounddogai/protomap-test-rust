use super::*;

use acme_kit::text;

/// Where a scheduled rollback score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRollbackScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledRollbackScoreStatus {
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

impl std::fmt::Display for ScheduledRollbackScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled rollback score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackScore {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub fingerprint: String,
    pub priority: i64,
    pub status: ScheduledRollbackScoreStatus,
}

impl ScheduledRollbackScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRollbackScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRollbackScoreStatus) -> bool {
        match self.status {
            ScheduledRollbackScoreStatus::Open => {
                return next == ScheduledRollbackScoreStatus::InReview
                    || next == ScheduledRollbackScoreStatus::Reopened;
            }
            ScheduledRollbackScoreStatus::InReview => {
                return next == ScheduledRollbackScoreStatus::Resolved
                    || next == ScheduledRollbackScoreStatus::Archived;
            }
            ScheduledRollbackScoreStatus::Resolved => {
                match next {
                    ScheduledRollbackScoreStatus::Reopened | ScheduledRollbackScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRollbackScoreStatus::Reopened => {
                return next == ScheduledRollbackScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRollbackScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rollback score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.capacity as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRollbackScore {
        ScheduledRollbackScore {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            fingerprint: self.fingerprint.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled rollback score in status open.
pub fn new_scheduled_rollback_score(code: &str, label: &str) -> ScheduledRollbackScore {
    ScheduledRollbackScore {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        fingerprint: "".to_string(),
        priority: 0,
        status: ScheduledRollbackScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_rollback_score("code-439", "label-496");
        assert_eq!(item.status, ScheduledRollbackScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_rollback_score("code-439", "label-496");
        if item.move_to(ScheduledRollbackScoreStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRollbackScoreStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_rollback_score("code-439", "label-496");
        item.status = ScheduledRollbackScoreStatus::Archived;
        assert!(
            item.move_to(ScheduledRollbackScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
