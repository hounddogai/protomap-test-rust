use super::*;

use acme_kit::text;

/// Where a audit event rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRollup {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub scope: String,
    pub duration_minutes: i64,
    pub amount: f64,
    pub source: String,
    pub status: AuditEventRollupStatus,
}

impl AuditEventRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventRollupStatus) -> bool {
        match self.status {
            AuditEventRollupStatus::Proposed => {
                match next {
                    AuditEventRollupStatus::Rejected | AuditEventRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventRollupStatus::Approved => {
                return next == AuditEventRollupStatus::Rejected;
            }
            AuditEventRollupStatus::Rejected => {
                match next {
                    AuditEventRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AuditEventRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "audit event rollup", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.duration_minutes as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventRollup {
        AuditEventRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            scope: self.scope.clone(),
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a audit event rollup in status proposed.
pub fn new_audit_event_rollup(code: &str, title: &str) -> AuditEventRollup {
    AuditEventRollup {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        scope: "".to_string(),
        duration_minutes: 0,
        amount: 0.0,
        source: "".to_string(),
        status: AuditEventRollupStatus::Proposed,
    }
}
