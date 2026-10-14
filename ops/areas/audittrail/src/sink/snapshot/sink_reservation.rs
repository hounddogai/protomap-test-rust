use super::*;

use acme_kit::text;

/// Where a sink reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SinkReservationStatus {
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

impl std::fmt::Display for SinkReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkReservation {
    pub code: String,
    pub label: String,
    pub source: String,
    pub currency: String,
    pub approved: bool,
    pub burst: i64,
    pub status: SinkReservationStatus,
}

impl SinkReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkReservationStatus) -> bool {
        match self.status {
            SinkReservationStatus::Idle => {
                return next == SinkReservationStatus::Ready;
            }
            SinkReservationStatus::Warming => {
                return next == SinkReservationStatus::Stopped;
            }
            SinkReservationStatus::Ready => {
                return next == SinkReservationStatus::Draining;
            }
            SinkReservationStatus::Draining => {
                return next == SinkReservationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "sink reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> SinkReservation {
        SinkReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            currency: self.currency.clone(),
            approved: self.approved,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a sink reservation in status idle.
pub fn new_sink_reservation(code: &str, label: &str) -> SinkReservation {
    SinkReservation {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        currency: "".to_string(),
        approved: false,
        burst: 0,
        status: SinkReservationStatus::Idle,
    }
}
