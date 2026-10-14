use super::*;

use acme_kit::text;

/// Where a pending identity checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingIdentityCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingIdentityCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingIdentityCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending identity checkpoint as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityCheckpoint {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub ends_at: i64,
    pub quantity: i64,
    pub country: String,
    pub ratio: f64,
    pub status: PendingIdentityCheckpointStatus,
}

impl PendingIdentityCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingIdentityCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingIdentityCheckpointStatus) -> bool {
        match self.status {
            PendingIdentityCheckpointStatus::Created => {
                match next {
                    PendingIdentityCheckpointStatus::Validated | PendingIdentityCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingIdentityCheckpointStatus::Validated => {
                match next {
                    PendingIdentityCheckpointStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingIdentityCheckpointStatus::Published => {
                match next {
                    PendingIdentityCheckpointStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingIdentityCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending identity checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        (self.ratio * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingIdentityCheckpoint {
        PendingIdentityCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            ends_at: self.ends_at,
            quantity: self.quantity,
            country: self.country.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a pending identity checkpoint in status created.
pub fn new_pending_identity_checkpoint(code: &str, name: &str) -> PendingIdentityCheckpoint {
    PendingIdentityCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        ends_at: 0,
        quantity: 0,
        country: "".to_string(),
        ratio: 0.0,
        status: PendingIdentityCheckpointStatus::Created,
    }
}
