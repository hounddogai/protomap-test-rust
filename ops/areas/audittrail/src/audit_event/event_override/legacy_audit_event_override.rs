use super::*;

use acme_kit::text;

/// Where a legacy audit event override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAuditEventOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyAuditEventOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LegacyAuditEventOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy audit event override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventOverride {
    pub code: String,
    pub name: String,
    pub country: String,
    pub maximum: f64,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub weight: f64,
    pub status: LegacyAuditEventOverrideStatus,
}

impl LegacyAuditEventOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAuditEventOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAuditEventOverrideStatus) -> bool {
        match self.status {
            LegacyAuditEventOverrideStatus::New => {
                return next == LegacyAuditEventOverrideStatus::Failed;
            }
            LegacyAuditEventOverrideStatus::Queued => {
                return next == LegacyAuditEventOverrideStatus::Failed;
            }
            LegacyAuditEventOverrideStatus::Running => {
                return next == LegacyAuditEventOverrideStatus::Failed;
            }
            LegacyAuditEventOverrideStatus::Done => {
                return next == LegacyAuditEventOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyAuditEventOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy audit event override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy audit event override", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.maximum * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAuditEventOverride {
        LegacyAuditEventOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            maximum: self.maximum,
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy audit event override in status new.
pub fn new_legacy_audit_event_override(code: &str, name: &str) -> LegacyAuditEventOverride {
    LegacyAuditEventOverride {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        maximum: 0.0,
        ttl_seconds: 0,
        ceiling: 0.0,
        weight: 0.0,
        status: LegacyAuditEventOverrideStatus::New,
    }
}
