use super::*;

use acme_kit::text;

/// Where a seasonal measurement allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalMeasurementAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalMeasurementAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalMeasurementAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal measurement allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementAllocation {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub visible: bool,
    pub unit: String,
    pub status: SeasonalMeasurementAllocationStatus,
}

impl SeasonalMeasurementAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalMeasurementAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalMeasurementAllocationStatus) -> bool {
        match self.status {
            SeasonalMeasurementAllocationStatus::Proposed => {
                return next == SeasonalMeasurementAllocationStatus::Rejected
                    || next == SeasonalMeasurementAllocationStatus::Withdrawn;
            }
            SeasonalMeasurementAllocationStatus::Approved => {
                match next {
                    SeasonalMeasurementAllocationStatus::Rejected | SeasonalMeasurementAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalMeasurementAllocationStatus::Rejected => {
                return next == SeasonalMeasurementAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalMeasurementAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal measurement allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> SeasonalMeasurementAllocation {
        SeasonalMeasurementAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            visible: self.visible,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal measurement allocation in status proposed.
pub fn new_seasonal_measurement_allocation(code: &str, label: &str) -> SeasonalMeasurementAllocation {
    SeasonalMeasurementAllocation {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        visible: false,
        unit: "".to_string(),
        status: SeasonalMeasurementAllocationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_measurement_allocation("code-479", "label-346");
        assert_eq!(item.status, SeasonalMeasurementAllocationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_seasonal_measurement_allocation("code-479", "label-346");
        if item.move_to(SeasonalMeasurementAllocationStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalMeasurementAllocationStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_measurement_allocation("code-479", "label-346");
        item.status = SeasonalMeasurementAllocationStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalMeasurementAllocationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
