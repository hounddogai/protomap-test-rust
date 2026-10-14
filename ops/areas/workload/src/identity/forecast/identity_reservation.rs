use super::*;

use acme_kit::text;

/// Where a identity reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IdentityReservationStatus {
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

impl std::fmt::Display for IdentityReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityReservation {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub country: String,
    pub quantity: i64,
    pub threshold: f64,
    pub region: String,
    pub timezone: String,
    pub status: IdentityReservationStatus,
}

impl IdentityReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityReservationStatus) -> bool {
        match self.status {
            IdentityReservationStatus::Idle => {
                return next == IdentityReservationStatus::Ready || next == IdentityReservationStatus::Stopped;
            }
            IdentityReservationStatus::Warming => {
                match next {
                    IdentityReservationStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityReservationStatus::Ready => {
                return next == IdentityReservationStatus::Draining;
            }
            IdentityReservationStatus::Draining => {
                match next {
                    IdentityReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: IdentityReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.threshold * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityReservation {
        IdentityReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            country: self.country.clone(),
            quantity: self.quantity,
            threshold: self.threshold,
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a identity reservation in status idle.
pub fn new_identity_reservation(code: &str, title: &str) -> IdentityReservation {
    IdentityReservation {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        country: "".to_string(),
        quantity: 0,
        threshold: 0.0,
        region: "".to_string(),
        timezone: "".to_string(),
        status: IdentityReservationStatus::Idle,
    }
}
