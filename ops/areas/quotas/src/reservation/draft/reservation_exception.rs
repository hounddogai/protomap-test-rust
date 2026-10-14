use super::*;

use acme_kit::text;

/// Where a reservation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationExceptionStatus {
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

impl std::fmt::Display for ReservationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation exception as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationException {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub ratio: f64,
    pub minimum: f64,
    pub external_ref: String,
    pub unit: String,
    pub status: ReservationExceptionStatus,
}

impl ReservationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationExceptionStatus) -> bool {
        match self.status {
            ReservationExceptionStatus::Idle => {
                match next {
                    ReservationExceptionStatus::Draining | ReservationExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationExceptionStatus::Warming => {
                return next == ReservationExceptionStatus::Stopped;
            }
            ReservationExceptionStatus::Ready => {
                return next == ReservationExceptionStatus::Draining || next == ReservationExceptionStatus::Stopped;
            }
            ReservationExceptionStatus::Draining => {
                return next == ReservationExceptionStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.minimum * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationException {
        ReservationException {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            ratio: self.ratio,
            minimum: self.minimum,
            external_ref: self.external_ref.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a reservation exception in status idle.
pub fn new_reservation_exception(code: &str, name: &str) -> ReservationException {
    ReservationException {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        ratio: 0.0,
        minimum: 0.0,
        external_ref: "".to_string(),
        unit: "".to_string(),
        status: ReservationExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_reservation_exception("code-422", "name-954");
        assert_eq!(item.status, ReservationExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_reservation_exception("code-422", "name-954");
        if item.move_to(ReservationExceptionStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationExceptionStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_reservation_exception("code-422", "name-954");
        item.status = ReservationExceptionStatus::Stopped;
        assert!(
            item.move_to(ReservationExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
