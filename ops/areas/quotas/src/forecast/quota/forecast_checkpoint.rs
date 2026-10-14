use super::*;

use acme_kit::text;

/// Where a forecast checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastCheckpointStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ForecastCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ForecastCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpoint {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub weight: f64,
    pub external_ref: String,
    pub status: ForecastCheckpointStatus,
}

impl ForecastCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastCheckpointStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastCheckpointStatus) -> bool {
        match self.status {
            ForecastCheckpointStatus::Idle => {
                match next {
                    ForecastCheckpointStatus::Warming | ForecastCheckpointStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastCheckpointStatus::Warming => {
                return next == ForecastCheckpointStatus::Draining || next == ForecastCheckpointStatus::Stopped;
            }
            ForecastCheckpointStatus::Ready => {
                match next {
                    ForecastCheckpointStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastCheckpointStatus::Draining => {
                match next {
                    ForecastCheckpointStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ForecastCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> ForecastCheckpoint {
        ForecastCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            weight: self.weight,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a forecast checkpoint in status idle.
pub fn new_forecast_checkpoint(code: &str, label: &str) -> ForecastCheckpoint {
    ForecastCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        weight: 0.0,
        external_ref: "".to_string(),
        status: ForecastCheckpointStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_forecast_checkpoint("code-190", "label-320");
        assert_eq!(item.status, ForecastCheckpointStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_forecast_checkpoint("code-190", "label-320");
        if item.move_to(ForecastCheckpointStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastCheckpointStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_forecast_checkpoint("code-190", "label-320");
        item.status = ForecastCheckpointStatus::Stopped;
        assert!(
            item.move_to(ForecastCheckpointStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
