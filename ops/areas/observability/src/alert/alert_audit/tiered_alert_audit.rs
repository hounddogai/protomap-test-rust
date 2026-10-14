use super::*;

use acme_kit::text;

/// Where a tiered alert audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAlertAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredAlertAuditStatus {
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

impl std::fmt::Display for TieredAlertAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered alert audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAudit {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub created_at: i64,
    pub capacity: i64,
    pub scope: String,
    pub timezone: String,
    pub quota_units: i64,
    pub source: String,
    pub status: TieredAlertAuditStatus,
}

impl TieredAlertAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAlertAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAlertAuditStatus) -> bool {
        match self.status {
            TieredAlertAuditStatus::Open => {
                return next == TieredAlertAuditStatus::Resolved || next == TieredAlertAuditStatus::Reopened;
            }
            TieredAlertAuditStatus::InReview => {
                return next == TieredAlertAuditStatus::Reopened || next == TieredAlertAuditStatus::Archived;
            }
            TieredAlertAuditStatus::Resolved => {
                match next {
                    TieredAlertAuditStatus::Reopened | TieredAlertAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAlertAuditStatus::Reopened => {
                match next {
                    TieredAlertAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredAlertAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered alert audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.starts_at as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAlertAudit {
        TieredAlertAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            created_at: self.created_at,
            capacity: self.capacity,
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            quota_units: self.quota_units,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered alert audit in status open.
pub fn new_tiered_alert_audit(code: &str, name: &str) -> TieredAlertAudit {
    TieredAlertAudit {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        created_at: 0,
        capacity: 0,
        scope: "".to_string(),
        timezone: "".to_string(),
        quota_units: 0,
        source: "".to_string(),
        status: TieredAlertAuditStatus::Open,
    }
}
