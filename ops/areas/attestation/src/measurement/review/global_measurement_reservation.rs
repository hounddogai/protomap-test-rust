use super::*;

use acme_kit::text;

/// Where a global measurement reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalMeasurementReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalMeasurementReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalMeasurementReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global measurement reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementReservation {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub grace_days: i64,
    pub discount: f64,
    pub status: GlobalMeasurementReservationStatus,
}

impl GlobalMeasurementReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalMeasurementReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalMeasurementReservationStatus) -> bool {
        match self.status {
            GlobalMeasurementReservationStatus::Created => {
                return next == GlobalMeasurementReservationStatus::Validated
                    || next == GlobalMeasurementReservationStatus::Retired;
            }
            GlobalMeasurementReservationStatus::Validated => {
                return next == GlobalMeasurementReservationStatus::Retired;
            }
            GlobalMeasurementReservationStatus::Published => {
                return next == GlobalMeasurementReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalMeasurementReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global measurement reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global measurement reservation", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.weight - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalMeasurementReservation {
        GlobalMeasurementReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            grace_days: self.grace_days,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global measurement reservation in status created.
pub fn new_global_measurement_reservation(code: &str, label: &str) -> GlobalMeasurementReservation {
    GlobalMeasurementReservation {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        grace_days: 0,
        discount: 0.0,
        status: GlobalMeasurementReservationStatus::Created,
    }
}
