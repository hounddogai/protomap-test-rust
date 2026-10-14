use super::*;

use acme_kit::text;

/// Where a regional usage audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalUsageAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalUsageAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalUsageAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional usage audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageAudit {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub contact_email: String,
    pub visible: bool,
    pub issuer: String,
    pub quantity: i64,
    pub status: RegionalUsageAuditStatus,
}

impl RegionalUsageAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalUsageAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalUsageAuditStatus) -> bool {
        match self.status {
            RegionalUsageAuditStatus::Requested => {
                return next == RegionalUsageAuditStatus::Revoked || next == RegionalUsageAuditStatus::Expired;
            }
            RegionalUsageAuditStatus::Granted => {
                match next {
                    RegionalUsageAuditStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageAuditStatus::Revoked => {
                match next {
                    RegionalUsageAuditStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalUsageAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional usage audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> RegionalUsageAudit {
        RegionalUsageAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            contact_email: self.contact_email.clone(),
            visible: self.visible,
            issuer: self.issuer.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional usage audit in status requested.
pub fn new_regional_usage_audit(code: &str, title: &str) -> RegionalUsageAudit {
    RegionalUsageAudit {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        contact_email: "".to_string(),
        visible: false,
        issuer: "".to_string(),
        quantity: 0,
        status: RegionalUsageAuditStatus::Requested,
    }
}
