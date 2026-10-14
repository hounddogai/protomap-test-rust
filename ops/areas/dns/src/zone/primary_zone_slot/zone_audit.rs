use super::*;

use acme_kit::text;

/// Where a zone audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ZoneAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ZoneAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAudit {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub threshold: f64,
    pub title: String,
    pub attempts: i64,
    pub timezone: String,
    pub sequence: i64,
    pub grace_days: i64,
    pub currency: String,
    pub status: ZoneAuditStatus,
}

impl ZoneAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneAuditStatus) -> bool {
        match self.status {
            ZoneAuditStatus::Requested => {
                match next {
                    ZoneAuditStatus::Granted | ZoneAuditStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAuditStatus::Granted => {
                match next {
                    ZoneAuditStatus::Revoked | ZoneAuditStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAuditStatus::Revoked => {
                match next {
                    ZoneAuditStatus::Expired => {
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

    pub fn move_to(&mut self, next: ZoneAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "zone audit", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.sequence as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneAudit {
        ZoneAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            threshold: self.threshold,
            title: self.title.clone(),
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            sequence: self.sequence,
            grace_days: self.grace_days,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a zone audit in status requested.
pub fn new_zone_audit(code: &str, label: &str) -> ZoneAudit {
    ZoneAudit {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        threshold: 0.0,
        title: "".to_string(),
        attempts: 0,
        timezone: "".to_string(),
        sequence: 0,
        grace_days: 0,
        currency: "".to_string(),
        status: ZoneAuditStatus::Requested,
    }
}
