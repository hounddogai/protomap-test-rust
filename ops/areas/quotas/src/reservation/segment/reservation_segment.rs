use super::*;

use acme_kit::text;

/// Where a reservation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationSegmentStatus {
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

impl std::fmt::Display for ReservationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation segment as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSegment {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub maximum: f64,
    pub summary: String,
    pub minimum: f64,
    pub status: ReservationSegmentStatus,
}

impl ReservationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationSegmentStatus) -> bool {
        match self.status {
            ReservationSegmentStatus::Idle => {
                return next == ReservationSegmentStatus::Stopped;
            }
            ReservationSegmentStatus::Warming => {
                return next == ReservationSegmentStatus::Draining || next == ReservationSegmentStatus::Stopped;
            }
            ReservationSegmentStatus::Ready => {
                return next == ReservationSegmentStatus::Draining;
            }
            ReservationSegmentStatus::Draining => {
                return next == ReservationSegmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.maximum - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationSegment {
        ReservationSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            summary: self.summary.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a reservation segment in status idle.
pub fn new_reservation_segment(code: &str, name: &str) -> ReservationSegment {
    ReservationSegment {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        maximum: 0.0,
        summary: "".to_string(),
        minimum: 0.0,
        status: ReservationSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_reservation_segment("code-70", "name-688");
        assert_eq!(item.status, ReservationSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_reservation_segment("code-70", "name-688");
        if item.move_to(ReservationSegmentStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationSegmentStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_reservation_segment("code-70", "name-688");
        item.status = ReservationSegmentStatus::Stopped;
        assert!(
            item.move_to(ReservationSegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
