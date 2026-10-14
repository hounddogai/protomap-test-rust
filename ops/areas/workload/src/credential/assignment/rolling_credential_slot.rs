use super::*;

use acme_kit::text;

/// Where a rolling credential slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingCredentialSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingCredentialSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingCredentialSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling credential slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSlot {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub key_id: String,
    pub label: String,
    pub weight: f64,
    pub ttl_seconds: i64,
    pub status: RollingCredentialSlotStatus,
}

impl RollingCredentialSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingCredentialSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingCredentialSlotStatus) -> bool {
        match self.status {
            RollingCredentialSlotStatus::Proposed => {
                return next == RollingCredentialSlotStatus::Approved || next == RollingCredentialSlotStatus::Withdrawn;
            }
            RollingCredentialSlotStatus::Approved => {
                return next == RollingCredentialSlotStatus::Rejected || next == RollingCredentialSlotStatus::Withdrawn;
            }
            RollingCredentialSlotStatus::Rejected => {
                return next == RollingCredentialSlotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingCredentialSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling credential slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling credential slot", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingCredentialSlot {
        RollingCredentialSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            key_id: self.key_id.clone(),
            label: self.label.clone(),
            weight: self.weight,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling credential slot in status proposed.
pub fn new_rolling_credential_slot(code: &str, name: &str) -> RollingCredentialSlot {
    RollingCredentialSlot {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        key_id: "".to_string(),
        label: "".to_string(),
        weight: 0.0,
        ttl_seconds: 0,
        status: RollingCredentialSlotStatus::Proposed,
    }
}
