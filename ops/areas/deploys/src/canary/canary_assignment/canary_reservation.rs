use super::*;

use acme_kit::text;

/// Where a canary reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CanaryReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for CanaryReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryReservation {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub window_days: i64,
    pub approved: bool,
    pub key_id: String,
    pub status: CanaryReservationStatus,
}

impl CanaryReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryReservationStatus) -> bool {
        match self.status {
            CanaryReservationStatus::Open => {
                match next {
                    CanaryReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryReservationStatus::InReview => {
                return next == CanaryReservationStatus::Resolved || next == CanaryReservationStatus::Archived;
            }
            CanaryReservationStatus::Resolved => {
                return next == CanaryReservationStatus::Reopened || next == CanaryReservationStatus::Archived;
            }
            CanaryReservationStatus::Reopened => {
                match next {
                    CanaryReservationStatus::Archived => {
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

    pub fn move_to(&mut self, next: CanaryReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryReservation {
        CanaryReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            window_days: self.window_days,
            approved: self.approved,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a canary reservation in status open.
pub fn new_canary_reservation(code: &str, label: &str) -> CanaryReservation {
    CanaryReservation {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        window_days: 0,
        approved: false,
        key_id: "".to_string(),
        status: CanaryReservationStatus::Open,
    }
}
