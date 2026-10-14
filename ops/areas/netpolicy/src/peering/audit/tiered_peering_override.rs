use super::*;

use acme_kit::text;

/// Where a tiered peering override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPeeringOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredPeeringOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TieredPeeringOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered peering override as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverride {
    pub code: String,
    pub label: String,
    pub source: String,
    pub scope: String,
    pub capacity: i64,
    pub category: String,
    pub discount: f64,
    pub duration_minutes: i64,
    pub description: String,
    pub status: TieredPeeringOverrideStatus,
}

impl TieredPeeringOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPeeringOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPeeringOverrideStatus) -> bool {
        match self.status {
            TieredPeeringOverrideStatus::New => {
                match next {
                    TieredPeeringOverrideStatus::Queued | TieredPeeringOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPeeringOverrideStatus::Queued => {
                match next {
                    TieredPeeringOverrideStatus::Done | TieredPeeringOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPeeringOverrideStatus::Running => {
                match next {
                    TieredPeeringOverrideStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPeeringOverrideStatus::Done => {
                match next {
                    TieredPeeringOverrideStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredPeeringOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered peering override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.capacity as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPeeringOverride {
        TieredPeeringOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            scope: self.scope.clone(),
            capacity: self.capacity,
            category: self.category.clone(),
            discount: self.discount,
            duration_minutes: self.duration_minutes,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered peering override in status new.
pub fn new_tiered_peering_override(code: &str, label: &str) -> TieredPeeringOverride {
    TieredPeeringOverride {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        scope: "".to_string(),
        capacity: 0,
        category: "".to_string(),
        discount: 0.0,
        duration_minutes: 0,
        description: "".to_string(),
        status: TieredPeeringOverrideStatus::New,
    }
}
