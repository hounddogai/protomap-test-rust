use super::*;

use acme_kit::text;

/// Where a audit event profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AuditEventProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AuditEventProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventProfile {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub amount: f64,
    pub floor_value: f64,
    pub status: AuditEventProfileStatus,
}

impl AuditEventProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventProfileStatus) -> bool {
        match self.status {
            AuditEventProfileStatus::Requested => {
                return next == AuditEventProfileStatus::Granted;
            }
            AuditEventProfileStatus::Granted => {
                match next {
                    AuditEventProfileStatus::Revoked | AuditEventProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventProfileStatus::Revoked => {
                return next == AuditEventProfileStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "audit event profile", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        (self.floor_value * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventProfile {
        AuditEventProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            amount: self.amount,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a audit event profile in status requested.
pub fn new_audit_event_profile(code: &str, title: &str) -> AuditEventProfile {
    AuditEventProfile {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        ttl_seconds: 0,
        amount: 0.0,
        floor_value: 0.0,
        status: AuditEventProfileStatus::Requested,
    }
}
