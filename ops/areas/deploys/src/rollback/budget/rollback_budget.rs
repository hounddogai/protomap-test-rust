use super::*;

use acme_kit::text;

/// Where a rollback budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollbackBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RollbackBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBudget {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub score: f64,
    pub burst: i64,
    pub fingerprint: String,
    pub tenant_id: String,
    pub port: i64,
    pub notes: String,
    pub status: RollbackBudgetStatus,
}

impl RollbackBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackBudgetStatus) -> bool {
        match self.status {
            RollbackBudgetStatus::Idle => {
                match next {
                    RollbackBudgetStatus::Warming | RollbackBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackBudgetStatus::Warming => {
                match next {
                    RollbackBudgetStatus::Draining | RollbackBudgetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackBudgetStatus::Ready => {
                return next == RollbackBudgetStatus::Draining;
            }
            RollbackBudgetStatus::Draining => {
                match next {
                    RollbackBudgetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollbackBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.port as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackBudget {
        RollbackBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            score: self.score,
            burst: self.burst,
            fingerprint: self.fingerprint.clone(),
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback budget in status idle.
pub fn new_rollback_budget(code: &str, label: &str) -> RollbackBudget {
    RollbackBudget {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        score: 0.0,
        burst: 0,
        fingerprint: "".to_string(),
        tenant_id: "".to_string(),
        port: 0,
        notes: "".to_string(),
        status: RollbackBudgetStatus::Idle,
    }
}
