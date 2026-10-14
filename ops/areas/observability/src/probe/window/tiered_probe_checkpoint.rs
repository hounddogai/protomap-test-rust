use super::*;

use acme_kit::text;

/// Where a tiered probe checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredProbeCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredProbeCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredProbeCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered probe checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeCheckpoint {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub quota_units: i64,
    pub minimum: f64,
    pub approved: bool,
    pub limit: i64,
    pub status: TieredProbeCheckpointStatus,
}

impl TieredProbeCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredProbeCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredProbeCheckpointStatus) -> bool {
        match self.status {
            TieredProbeCheckpointStatus::Proposed => {
                match next {
                    TieredProbeCheckpointStatus::Approved | TieredProbeCheckpointStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredProbeCheckpointStatus::Approved => {
                match next {
                    TieredProbeCheckpointStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredProbeCheckpointStatus::Rejected => {
                return next == TieredProbeCheckpointStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredProbeCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered probe checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered probe checkpoint", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.minimum - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredProbeCheckpoint {
        TieredProbeCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            quota_units: self.quota_units,
            minimum: self.minimum,
            approved: self.approved,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered probe checkpoint in status proposed.
pub fn new_tiered_probe_checkpoint(code: &str, name: &str) -> TieredProbeCheckpoint {
    TieredProbeCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        quota_units: 0,
        minimum: 0.0,
        approved: false,
        limit: 0,
        status: TieredProbeCheckpointStatus::Proposed,
    }
}
