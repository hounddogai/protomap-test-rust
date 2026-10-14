use super::*;

use acme_kit::text;

/// Where a measurement request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MeasurementRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MeasurementRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRequest {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub pinned: bool,
    pub duration_minutes: i64,
    pub quota_units: i64,
    pub country: String,
    pub status: MeasurementRequestStatus,
}

impl MeasurementRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementRequestStatus) -> bool {
        match self.status {
            MeasurementRequestStatus::Proposed => {
                match next {
                    MeasurementRequestStatus::Approved | MeasurementRequestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementRequestStatus::Approved => {
                match next {
                    MeasurementRequestStatus::Rejected | MeasurementRequestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementRequestStatus::Rejected => {
                match next {
                    MeasurementRequestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: MeasurementRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement request", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.quota_units as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementRequest {
        MeasurementRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            pinned: self.pinned,
            duration_minutes: self.duration_minutes,
            quota_units: self.quota_units,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a measurement request in status proposed.
pub fn new_measurement_request(code: &str, name: &str) -> MeasurementRequest {
    MeasurementRequest {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        pinned: false,
        duration_minutes: 0,
        quota_units: 0,
        country: "".to_string(),
        status: MeasurementRequestStatus::Proposed,
    }
}
