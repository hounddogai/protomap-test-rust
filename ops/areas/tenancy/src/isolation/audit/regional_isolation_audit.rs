use super::*;

use acme_kit::text;

/// Where a regional isolation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalIsolationAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalIsolationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalIsolationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional isolation audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIsolationAudit {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub description: String,
    pub capacity: i64,
    pub quantity: i64,
    pub status: RegionalIsolationAuditStatus,
}

impl RegionalIsolationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalIsolationAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalIsolationAuditStatus) -> bool {
        match self.status {
            RegionalIsolationAuditStatus::Requested => {
                return next == RegionalIsolationAuditStatus::Granted;
            }
            RegionalIsolationAuditStatus::Granted => {
                return next == RegionalIsolationAuditStatus::Revoked;
            }
            RegionalIsolationAuditStatus::Revoked => {
                match next {
                    RegionalIsolationAuditStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalIsolationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional isolation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quantity as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalIsolationAudit {
        RegionalIsolationAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            description: self.description.clone(),
            capacity: self.capacity,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional isolation audit in status requested.
pub fn new_regional_isolation_audit(code: &str, name: &str) -> RegionalIsolationAudit {
    RegionalIsolationAudit {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        description: "".to_string(),
        capacity: 0,
        quantity: 0,
        status: RegionalIsolationAuditStatus::Requested,
    }
}
