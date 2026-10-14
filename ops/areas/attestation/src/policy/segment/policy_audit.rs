use super::*;

use acme_kit::text;

/// Where a policy audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PolicyAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PolicyAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAudit {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub active: bool,
    pub attempts: i64,
    pub batch_size: i64,
    pub maximum: f64,
    pub minimum: f64,
    pub threshold: f64,
    pub status: PolicyAuditStatus,
}

impl PolicyAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyAuditStatus) -> bool {
        match self.status {
            PolicyAuditStatus::Requested => {
                match next {
                    PolicyAuditStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyAuditStatus::Granted => {
                return next == PolicyAuditStatus::Revoked || next == PolicyAuditStatus::Expired;
            }
            PolicyAuditStatus::Revoked => {
                return next == PolicyAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.minimum + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyAudit {
        PolicyAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            active: self.active,
            attempts: self.attempts,
            batch_size: self.batch_size,
            maximum: self.maximum,
            minimum: self.minimum,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a policy audit in status requested.
pub fn new_policy_audit(code: &str, name: &str) -> PolicyAudit {
    PolicyAudit {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        active: false,
        attempts: 0,
        batch_size: 0,
        maximum: 0.0,
        minimum: 0.0,
        threshold: 0.0,
        status: PolicyAuditStatus::Requested,
    }
}
