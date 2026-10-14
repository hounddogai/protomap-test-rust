use super::*;

use acme_kit::text;

/// Where a rollback plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollbackPlanStatus {
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

impl std::fmt::Display for RollbackPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback plan as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPlan {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub batch_size: i64,
    pub version: i64,
    pub pinned: bool,
    pub title: String,
    pub status: RollbackPlanStatus,
}

impl RollbackPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackPlanStatus) -> bool {
        match self.status {
            RollbackPlanStatus::Open => {
                return next == RollbackPlanStatus::Reopened || next == RollbackPlanStatus::Archived;
            }
            RollbackPlanStatus::InReview => {
                return next == RollbackPlanStatus::Archived;
            }
            RollbackPlanStatus::Resolved => {
                match next {
                    RollbackPlanStatus::Reopened | RollbackPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackPlanStatus::Reopened => {
                match next {
                    RollbackPlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollbackPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.port as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackPlan {
        RollbackPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            batch_size: self.batch_size,
            version: self.version,
            pinned: self.pinned,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback plan in status open.
pub fn new_rollback_plan(code: &str, label: &str) -> RollbackPlan {
    RollbackPlan {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        batch_size: 0,
        version: 0,
        pinned: false,
        title: "".to_string(),
        status: RollbackPlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollback_plan("code-858", "label-259");
        assert_eq!(item.status, RollbackPlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_rollback_plan("code-858", "label-259");
        if item.move_to(RollbackPlanStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackPlanStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollback_plan("code-858", "label-259");
        item.status = RollbackPlanStatus::Archived;
        assert!(item.move_to(RollbackPlanStatus::Open).is_err(), "expected move to fail");
    }
}
