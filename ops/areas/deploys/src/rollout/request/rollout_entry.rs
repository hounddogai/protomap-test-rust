use super::*;

use acme_kit::text;

/// Where a rollout entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RolloutEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RolloutEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEntry {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub priority: i64,
    pub unit: String,
    pub status: RolloutEntryStatus,
}

impl RolloutEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutEntryStatus) -> bool {
        match self.status {
            RolloutEntryStatus::Created => {
                match next {
                    RolloutEntryStatus::Published | RolloutEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutEntryStatus::Validated => {
                return next == RolloutEntryStatus::Published || next == RolloutEntryStatus::Retired;
            }
            RolloutEntryStatus::Published => {
                match next {
                    RolloutEntryStatus::Retired => {
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

    pub fn move_to(&mut self, next: RolloutEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.priority as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutEntry {
        RolloutEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            priority: self.priority,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rollout entry in status created.
pub fn new_rollout_entry(code: &str, name: &str) -> RolloutEntry {
    RolloutEntry {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        priority: 0,
        unit: "".to_string(),
        status: RolloutEntryStatus::Created,
    }
}
