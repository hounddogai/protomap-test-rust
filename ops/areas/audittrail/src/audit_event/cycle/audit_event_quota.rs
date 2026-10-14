use super::*;

use acme_kit::text;

/// Where a audit event quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AuditEventQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AuditEventQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventQuota {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub approved: bool,
    pub segment: String,
    pub name: String,
    pub starts_at: i64,
    pub retries: i64,
    pub contact_email: String,
    pub status: AuditEventQuotaStatus,
}

impl AuditEventQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventQuotaStatus) -> bool {
        match self.status {
            AuditEventQuotaStatus::Draft => {
                return next == AuditEventQuotaStatus::Pending;
            }
            AuditEventQuotaStatus::Pending => {
                return next == AuditEventQuotaStatus::Active;
            }
            AuditEventQuotaStatus::Active => {
                return next == AuditEventQuotaStatus::Closed;
            }
            AuditEventQuotaStatus::Suspended => {
                match next {
                    AuditEventQuotaStatus::Closed => {
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

    pub fn move_to(&mut self, next: AuditEventQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event quota {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.retries as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventQuota {
        AuditEventQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            approved: self.approved,
            segment: self.segment.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a audit event quota in status draft.
pub fn new_audit_event_quota(code: &str, label: &str) -> AuditEventQuota {
    AuditEventQuota {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        approved: false,
        segment: "".to_string(),
        name: "".to_string(),
        starts_at: 0,
        retries: 0,
        contact_email: "".to_string(),
        status: AuditEventQuotaStatus::Draft,
    }
}
