use super::*;

use acme_kit::text;

/// Where a rollout cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RolloutCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RolloutCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout cycle as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycle {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub timezone: String,
    pub pinned: bool,
    pub limit: i64,
    pub capacity: i64,
    pub segment: String,
    pub weight: f64,
    pub status: RolloutCycleStatus,
}

impl RolloutCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutCycleStatus) -> bool {
        match self.status {
            RolloutCycleStatus::Created => {
                match next {
                    RolloutCycleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutCycleStatus::Validated => {
                match next {
                    RolloutCycleStatus::Published | RolloutCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutCycleStatus::Published => {
                return next == RolloutCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.limit as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutCycle {
        RolloutCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            timezone: self.timezone.clone(),
            pinned: self.pinned,
            limit: self.limit,
            capacity: self.capacity,
            segment: self.segment.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rollout cycle in status created.
pub fn new_rollout_cycle(code: &str, label: &str) -> RolloutCycle {
    RolloutCycle {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        timezone: "".to_string(),
        pinned: false,
        limit: 0,
        capacity: 0,
        segment: "".to_string(),
        weight: 0.0,
        status: RolloutCycleStatus::Created,
    }
}
