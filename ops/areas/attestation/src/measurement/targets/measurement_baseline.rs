use super::*;

use acme_kit::text;

/// Where a measurement baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MeasurementBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MeasurementBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement baseline as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBaseline {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub unit: String,
    pub active: bool,
    pub status: MeasurementBaselineStatus,
}

impl MeasurementBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementBaselineStatus) -> bool {
        match self.status {
            MeasurementBaselineStatus::Created => {
                return next == MeasurementBaselineStatus::Published || next == MeasurementBaselineStatus::Retired;
            }
            MeasurementBaselineStatus::Validated => {
                match next {
                    MeasurementBaselineStatus::Published | MeasurementBaselineStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementBaselineStatus::Published => {
                match next {
                    MeasurementBaselineStatus::Retired => {
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

    pub fn move_to(&mut self, next: MeasurementBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> MeasurementBaseline {
        MeasurementBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            unit: self.unit.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a measurement baseline in status created.
pub fn new_measurement_baseline(code: &str, name: &str) -> MeasurementBaseline {
    MeasurementBaseline {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        unit: "".to_string(),
        active: false,
        status: MeasurementBaselineStatus::Created,
    }
}
