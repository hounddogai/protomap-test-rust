use super::*;

use acme_kit::text;

/// Where a primary usage audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryUsageAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryUsageAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryUsageAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary usage audit as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAudit {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub source: String,
    pub principal: String,
    pub amount: f64,
    pub status: PrimaryUsageAuditStatus,
}

impl PrimaryUsageAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryUsageAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryUsageAuditStatus) -> bool {
        match self.status {
            PrimaryUsageAuditStatus::Requested => {
                return next == PrimaryUsageAuditStatus::Granted;
            }
            PrimaryUsageAuditStatus::Granted => {
                match next {
                    PrimaryUsageAuditStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryUsageAuditStatus::Revoked => {
                return next == PrimaryUsageAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryUsageAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary usage audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary usage audit", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> PrimaryUsageAudit {
        PrimaryUsageAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            source: self.source.clone(),
            principal: self.principal.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary usage audit in status requested.
pub fn new_primary_usage_audit(code: &str, name: &str) -> PrimaryUsageAudit {
    PrimaryUsageAudit {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        source: "".to_string(),
        principal: "".to_string(),
        amount: 0.0,
        status: PrimaryUsageAuditStatus::Requested,
    }
}
