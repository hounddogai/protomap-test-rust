use super::*;

use acme_kit::text;

/// Where a pending retention rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRetentionRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingRetentionRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingRetentionRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending retention rollup as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionRollup {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub maximum: f64,
    pub visible: bool,
    pub description: String,
    pub budget: f64,
    pub status: PendingRetentionRollupStatus,
}

impl PendingRetentionRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRetentionRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRetentionRollupStatus) -> bool {
        match self.status {
            PendingRetentionRollupStatus::Created => {
                return next == PendingRetentionRollupStatus::Validated
                    || next == PendingRetentionRollupStatus::Retired;
            }
            PendingRetentionRollupStatus::Validated => {
                return next == PendingRetentionRollupStatus::Retired;
            }
            PendingRetentionRollupStatus::Published => {
                return next == PendingRetentionRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRetentionRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending retention rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.budget - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRetentionRollup {
        PendingRetentionRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            maximum: self.maximum,
            visible: self.visible,
            description: self.description.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending retention rollup in status created.
pub fn new_pending_retention_rollup(code: &str, name: &str) -> PendingRetentionRollup {
    PendingRetentionRollup {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        maximum: 0.0,
        visible: false,
        description: "".to_string(),
        budget: 0.0,
        status: PendingRetentionRollupStatus::Created,
    }
}
