use super::*;

use acme_kit::text;

/// Where a tenant checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TenantCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TenantCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantCheckpoint {
    pub code: String,
    pub label: String,
    pub category: String,
    pub title: String,
    pub sequence: i64,
    pub quota_units: i64,
    pub unit: String,
    pub cipher: String,
    pub status: TenantCheckpointStatus,
}

impl TenantCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantCheckpointStatus) -> bool {
        match self.status {
            TenantCheckpointStatus::Requested => {
                match next {
                    TenantCheckpointStatus::Granted | TenantCheckpointStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantCheckpointStatus::Granted => {
                return next == TenantCheckpointStatus::Expired;
            }
            TenantCheckpointStatus::Revoked => {
                match next {
                    TenantCheckpointStatus::Expired => {
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

    pub fn move_to(&mut self, next: TenantCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.quota_units as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantCheckpoint {
        TenantCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            quota_units: self.quota_units,
            unit: self.unit.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tenant checkpoint in status requested.
pub fn new_tenant_checkpoint(code: &str, label: &str) -> TenantCheckpoint {
    TenantCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        title: "".to_string(),
        sequence: 0,
        quota_units: 0,
        unit: "".to_string(),
        cipher: "".to_string(),
        status: TenantCheckpointStatus::Requested,
    }
}
