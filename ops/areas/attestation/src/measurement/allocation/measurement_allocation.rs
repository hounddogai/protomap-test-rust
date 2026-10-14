use super::*;

use acme_kit::text;

/// Where a measurement allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MeasurementAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MeasurementAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One measurement allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAllocation {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub ceiling: f64,
    pub timezone: String,
    pub cost: f64,
    pub category: String,
    pub ratio: f64,
    pub unit: String,
    pub key_id: String,
    pub status: MeasurementAllocationStatus,
}

impl MeasurementAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementAllocationStatus) -> bool {
        match self.status {
            MeasurementAllocationStatus::Created => {
                match next {
                    MeasurementAllocationStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementAllocationStatus::Validated => {
                match next {
                    MeasurementAllocationStatus::Published | MeasurementAllocationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementAllocationStatus::Published => {
                match next {
                    MeasurementAllocationStatus::Retired => {
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

    pub fn move_to(&mut self, next: MeasurementAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.cost - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementAllocation {
        MeasurementAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            ceiling: self.ceiling,
            timezone: self.timezone.clone(),
            cost: self.cost,
            category: self.category.clone(),
            ratio: self.ratio,
            unit: self.unit.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a measurement allocation in status created.
pub fn new_measurement_allocation(code: &str, label: &str) -> MeasurementAllocation {
    MeasurementAllocation {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        ceiling: 0.0,
        timezone: "".to_string(),
        cost: 0.0,
        category: "".to_string(),
        ratio: 0.0,
        unit: "".to_string(),
        key_id: "".to_string(),
        status: MeasurementAllocationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_measurement_allocation("code-734", "label-955");
        assert_eq!(item.status, MeasurementAllocationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_measurement_allocation("code-734", "label-955");
        if item.move_to(MeasurementAllocationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementAllocationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_measurement_allocation("code-734", "label-955");
        item.status = MeasurementAllocationStatus::Retired;
        assert!(
            item.move_to(MeasurementAllocationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
