use super::*;

use acme_kit::text;

/// Where a weighted measurement group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedMeasurementGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedMeasurementGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedMeasurementGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted measurement group as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementGroup {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub ends_at: i64,
    pub cipher: String,
    pub status: WeightedMeasurementGroupStatus,
}

impl WeightedMeasurementGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedMeasurementGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedMeasurementGroupStatus) -> bool {
        match self.status {
            WeightedMeasurementGroupStatus::Proposed => {
                return next == WeightedMeasurementGroupStatus::Approved
                    || next == WeightedMeasurementGroupStatus::Rejected;
            }
            WeightedMeasurementGroupStatus::Approved => {
                match next {
                    WeightedMeasurementGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedMeasurementGroupStatus::Rejected => {
                match next {
                    WeightedMeasurementGroupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: WeightedMeasurementGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted measurement group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> WeightedMeasurementGroup {
        WeightedMeasurementGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            ends_at: self.ends_at,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted measurement group in status proposed.
pub fn new_weighted_measurement_group(code: &str, name: &str) -> WeightedMeasurementGroup {
    WeightedMeasurementGroup {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        ends_at: 0,
        cipher: "".to_string(),
        status: WeightedMeasurementGroupStatus::Proposed,
    }
}
