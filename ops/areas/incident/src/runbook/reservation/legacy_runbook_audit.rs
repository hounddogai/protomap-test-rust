use super::*;

use acme_kit::text;

/// Where a legacy runbook audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRunbookAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyRunbookAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyRunbookAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy runbook audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAudit {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub rate: f64,
    pub maximum: f64,
    pub grace_days: i64,
    pub archived: bool,
    pub host: String,
    pub currency: String,
    pub created_at: i64,
    pub locale: String,
    pub status: LegacyRunbookAuditStatus,
}

impl LegacyRunbookAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRunbookAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRunbookAuditStatus) -> bool {
        match self.status {
            LegacyRunbookAuditStatus::Requested => {
                match next {
                    LegacyRunbookAuditStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRunbookAuditStatus::Granted => {
                return next == LegacyRunbookAuditStatus::Revoked || next == LegacyRunbookAuditStatus::Expired;
            }
            LegacyRunbookAuditStatus::Revoked => {
                match next {
                    LegacyRunbookAuditStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyRunbookAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy runbook audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.grace_days as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRunbookAudit {
        LegacyRunbookAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            rate: self.rate,
            maximum: self.maximum,
            grace_days: self.grace_days,
            archived: self.archived,
            host: self.host.clone(),
            currency: self.currency.clone(),
            created_at: self.created_at,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy runbook audit in status requested.
pub fn new_legacy_runbook_audit(code: &str, name: &str) -> LegacyRunbookAudit {
    LegacyRunbookAudit {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        rate: 0.0,
        maximum: 0.0,
        grace_days: 0,
        archived: false,
        host: "".to_string(),
        currency: "".to_string(),
        created_at: 0,
        locale: "".to_string(),
        status: LegacyRunbookAuditStatus::Requested,
    }
}
