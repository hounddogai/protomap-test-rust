use super::*;

use acme_kit::text;

/// Where a audit event budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AuditEventBudgetStatus {
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

impl std::fmt::Display for AuditEventBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event budget as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBudget {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub quota_units: i64,
    pub tenant_id: String,
    pub starts_at: i64,
    pub status: AuditEventBudgetStatus,
}

impl AuditEventBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventBudgetStatus) -> bool {
        match self.status {
            AuditEventBudgetStatus::Draft => {
                return next == AuditEventBudgetStatus::Pending;
            }
            AuditEventBudgetStatus::Pending => {
                return next == AuditEventBudgetStatus::Suspended || next == AuditEventBudgetStatus::Closed;
            }
            AuditEventBudgetStatus::Active => {
                return next == AuditEventBudgetStatus::Suspended;
            }
            AuditEventBudgetStatus::Suspended => {
                return next == AuditEventBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event budget {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.quota_units as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventBudget {
        AuditEventBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            quota_units: self.quota_units,
            tenant_id: self.tenant_id.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a audit event budget in status draft.
pub fn new_audit_event_budget(code: &str, label: &str) -> AuditEventBudget {
    AuditEventBudget {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        quota_units: 0,
        tenant_id: "".to_string(),
        starts_at: 0,
        status: AuditEventBudgetStatus::Draft,
    }
}
