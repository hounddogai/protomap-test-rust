use super::*;

use acme_kit::text;

/// Where a capped rollback milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRollbackMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedRollbackMilestoneStatus {
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

impl std::fmt::Display for CappedRollbackMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped rollback milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackMilestone {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub port: i64,
    pub amount: f64,
    pub retries: i64,
    pub status: CappedRollbackMilestoneStatus,
}

impl CappedRollbackMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRollbackMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRollbackMilestoneStatus) -> bool {
        match self.status {
            CappedRollbackMilestoneStatus::Draft => {
                match next {
                    CappedRollbackMilestoneStatus::Active | CappedRollbackMilestoneStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRollbackMilestoneStatus::Pending => {
                return next == CappedRollbackMilestoneStatus::Active
                    || next == CappedRollbackMilestoneStatus::Suspended;
            }
            CappedRollbackMilestoneStatus::Active => {
                match next {
                    CappedRollbackMilestoneStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRollbackMilestoneStatus::Suspended => {
                return next == CappedRollbackMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRollbackMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped rollback milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped rollback milestone", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.amount - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRollbackMilestone {
        CappedRollbackMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            port: self.port,
            amount: self.amount,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped rollback milestone in status draft.
pub fn new_capped_rollback_milestone(code: &str, name: &str) -> CappedRollbackMilestone {
    CappedRollbackMilestone {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        port: 0,
        amount: 0.0,
        retries: 0,
        status: CappedRollbackMilestoneStatus::Draft,
    }
}
