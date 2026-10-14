use super::*;

use acme_kit::text;

/// Where a strict overage bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictOverageBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictOverageBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictOverageBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict overage bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageBundle {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub tenant_id: String,
    pub timezone: String,
    pub capacity: i64,
    pub limit: i64,
    pub country: String,
    pub checksum: String,
    pub cipher: String,
    pub status: StrictOverageBundleStatus,
}

impl StrictOverageBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictOverageBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictOverageBundleStatus) -> bool {
        match self.status {
            StrictOverageBundleStatus::Created => {
                return next == StrictOverageBundleStatus::Validated;
            }
            StrictOverageBundleStatus::Validated => {
                return next == StrictOverageBundleStatus::Published || next == StrictOverageBundleStatus::Retired;
            }
            StrictOverageBundleStatus::Published => {
                return next == StrictOverageBundleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictOverageBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict overage bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.capacity as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictOverageBundle {
        StrictOverageBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            tenant_id: self.tenant_id.clone(),
            timezone: self.timezone.clone(),
            capacity: self.capacity,
            limit: self.limit,
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict overage bundle in status created.
pub fn new_strict_overage_bundle(code: &str, label: &str) -> StrictOverageBundle {
    StrictOverageBundle {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        tenant_id: "".to_string(),
        timezone: "".to_string(),
        capacity: 0,
        limit: 0,
        country: "".to_string(),
        checksum: "".to_string(),
        cipher: "".to_string(),
        status: StrictOverageBundleStatus::Created,
    }
}
