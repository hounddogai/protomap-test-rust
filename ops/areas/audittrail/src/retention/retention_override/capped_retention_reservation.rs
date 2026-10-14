use super::*;

use acme_kit::text;

/// Where a capped retention reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRetentionReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedRetentionReservationStatus {
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

impl std::fmt::Display for CappedRetentionReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped retention reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionReservation {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub rate: f64,
    pub score: f64,
    pub status: CappedRetentionReservationStatus,
}

impl CappedRetentionReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRetentionReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRetentionReservationStatus) -> bool {
        match self.status {
            CappedRetentionReservationStatus::Idle => {
                match next {
                    CappedRetentionReservationStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRetentionReservationStatus::Warming => {
                return next == CappedRetentionReservationStatus::Draining
                    || next == CappedRetentionReservationStatus::Stopped;
            }
            CappedRetentionReservationStatus::Ready => {
                return next == CappedRetentionReservationStatus::Stopped;
            }
            CappedRetentionReservationStatus::Draining => {
                return next == CappedRetentionReservationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRetentionReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped retention reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped retention reservation", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        (self.rate * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRetentionReservation {
        CappedRetentionReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            rate: self.rate,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped retention reservation in status idle.
pub fn new_capped_retention_reservation(code: &str, name: &str) -> CappedRetentionReservation {
    CappedRetentionReservation {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        rate: 0.0,
        score: 0.0,
        status: CappedRetentionReservationStatus::Idle,
    }
}
