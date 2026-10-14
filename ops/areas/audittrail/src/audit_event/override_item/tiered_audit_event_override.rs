use super::*;

use acme_kit::text;

/// Where a tiered audit event override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAuditEventOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredAuditEventOverrideStatus {
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

impl std::fmt::Display for TieredAuditEventOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered audit event override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventOverride {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub approved: bool,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub status: TieredAuditEventOverrideStatus,
}

impl TieredAuditEventOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAuditEventOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAuditEventOverrideStatus) -> bool {
        match self.status {
            TieredAuditEventOverrideStatus::Open => {
                return next == TieredAuditEventOverrideStatus::Resolved
                    || next == TieredAuditEventOverrideStatus::Archived;
            }
            TieredAuditEventOverrideStatus::InReview => {
                match next {
                    TieredAuditEventOverrideStatus::Resolved | TieredAuditEventOverrideStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAuditEventOverrideStatus::Resolved => {
                return next == TieredAuditEventOverrideStatus::Reopened
                    || next == TieredAuditEventOverrideStatus::Archived;
            }
            TieredAuditEventOverrideStatus::Reopened => {
                match next {
                    TieredAuditEventOverrideStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredAuditEventOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered audit event override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered audit event override", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.expires_at as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAuditEventOverride {
        TieredAuditEventOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            approved: self.approved,
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered audit event override in status open.
pub fn new_tiered_audit_event_override(code: &str, label: &str) -> TieredAuditEventOverride {
    TieredAuditEventOverride {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        approved: false,
        duration_minutes: 0,
        window_days: 0,
        status: TieredAuditEventOverrideStatus::Open,
    }
}
