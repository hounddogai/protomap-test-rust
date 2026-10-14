use super::*;

use acme_kit::text;

/// Where a global silence target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSilenceTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalSilenceTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalSilenceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global silence target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceTarget {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub summary: String,
    pub capacity: i64,
    pub tenant_id: String,
    pub status: GlobalSilenceTargetStatus,
}

impl GlobalSilenceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSilenceTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSilenceTargetStatus) -> bool {
        match self.status {
            GlobalSilenceTargetStatus::Created => {
                return next == GlobalSilenceTargetStatus::Validated || next == GlobalSilenceTargetStatus::Published;
            }
            GlobalSilenceTargetStatus::Validated => {
                return next == GlobalSilenceTargetStatus::Published;
            }
            GlobalSilenceTargetStatus::Published => {
                return next == GlobalSilenceTargetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSilenceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global silence target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.capacity as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSilenceTarget {
        GlobalSilenceTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            summary: self.summary.clone(),
            capacity: self.capacity,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global silence target in status created.
pub fn new_global_silence_target(code: &str, name: &str) -> GlobalSilenceTarget {
    GlobalSilenceTarget {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        summary: "".to_string(),
        capacity: 0,
        tenant_id: "".to_string(),
        status: GlobalSilenceTargetStatus::Created,
    }
}
