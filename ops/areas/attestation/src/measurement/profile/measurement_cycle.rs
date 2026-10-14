use super::*;

use acme_kit::text;

/// Where a measurement cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MeasurementCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MeasurementCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One measurement cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycle {
    pub code: String,
    pub name: String,
    pub label: String,
    pub category: String,
    pub issuer: String,
    pub limit: i64,
    pub ratio: f64,
    pub status: MeasurementCycleStatus,
}

impl MeasurementCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementCycleStatus) -> bool {
        match self.status {
            MeasurementCycleStatus::Created => {
                return next == MeasurementCycleStatus::Validated || next == MeasurementCycleStatus::Retired;
            }
            MeasurementCycleStatus::Validated => {
                return next == MeasurementCycleStatus::Retired;
            }
            MeasurementCycleStatus::Published => {
                match next {
                    MeasurementCycleStatus::Retired => {
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

    pub fn move_to(&mut self, next: MeasurementCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ratio - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementCycle {
        MeasurementCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            limit: self.limit,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a measurement cycle in status created.
pub fn new_measurement_cycle(code: &str, name: &str) -> MeasurementCycle {
    MeasurementCycle {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        category: "".to_string(),
        issuer: "".to_string(),
        limit: 0,
        ratio: 0.0,
        status: MeasurementCycleStatus::Created,
    }
}
