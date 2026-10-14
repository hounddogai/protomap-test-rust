use super::*;

use acme_kit::text;

/// Where a weighted measurement reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedMeasurementReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedMeasurementReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for WeightedMeasurementReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted measurement reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementReservation {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub country: String,
    pub key_id: String,
    pub cost: f64,
    pub contact_email: String,
    pub rate: f64,
    pub batch_size: i64,
    pub status: WeightedMeasurementReservationStatus,
}

impl WeightedMeasurementReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedMeasurementReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedMeasurementReservationStatus) -> bool {
        match self.status {
            WeightedMeasurementReservationStatus::Planned => {
                match next {
                    WeightedMeasurementReservationStatus::Completed
                    | WeightedMeasurementReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedMeasurementReservationStatus::Scheduled => {
                match next {
                    WeightedMeasurementReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedMeasurementReservationStatus::InProgress => {
                return next == WeightedMeasurementReservationStatus::Completed
                    || next == WeightedMeasurementReservationStatus::Cancelled;
            }
            WeightedMeasurementReservationStatus::Completed => {
                match next {
                    WeightedMeasurementReservationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedMeasurementReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted measurement reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted measurement reservation", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.batch_size as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedMeasurementReservation {
        WeightedMeasurementReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            country: self.country.clone(),
            key_id: self.key_id.clone(),
            cost: self.cost,
            contact_email: self.contact_email.clone(),
            rate: self.rate,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted measurement reservation in status planned.
pub fn new_weighted_measurement_reservation(code: &str, title: &str) -> WeightedMeasurementReservation {
    WeightedMeasurementReservation {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        country: "".to_string(),
        key_id: "".to_string(),
        cost: 0.0,
        contact_email: "".to_string(),
        rate: 0.0,
        batch_size: 0,
        status: WeightedMeasurementReservationStatus::Planned,
    }
}
