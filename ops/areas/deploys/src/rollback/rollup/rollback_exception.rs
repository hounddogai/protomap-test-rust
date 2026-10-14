use super::*;

use acme_kit::text;

/// Where a rollback exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollbackExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollbackExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback exception as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackException {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub cost: f64,
    pub retries: i64,
    pub status: RollbackExceptionStatus,
}

impl RollbackException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackExceptionStatus) -> bool {
        match self.status {
            RollbackExceptionStatus::Proposed => {
                return next == RollbackExceptionStatus::Approved || next == RollbackExceptionStatus::Rejected;
            }
            RollbackExceptionStatus::Approved => {
                match next {
                    RollbackExceptionStatus::Rejected | RollbackExceptionStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackExceptionStatus::Rejected => {
                return next == RollbackExceptionStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.retries as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackException {
        RollbackException {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            cost: self.cost,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollback exception in status proposed.
pub fn new_rollback_exception(code: &str, name: &str) -> RollbackException {
    RollbackException {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        cost: 0.0,
        retries: 0,
        status: RollbackExceptionStatus::Proposed,
    }
}
