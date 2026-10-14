use super::*;

use acme_kit::text;

/// Where a pending canary reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingCanaryReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingCanaryReservationStatus {
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

impl std::fmt::Display for PendingCanaryReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending canary reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryReservation {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub ttl_seconds: i64,
    pub cost: f64,
    pub status: PendingCanaryReservationStatus,
}

impl PendingCanaryReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingCanaryReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingCanaryReservationStatus) -> bool {
        match self.status {
            PendingCanaryReservationStatus::Idle => {
                match next {
                    PendingCanaryReservationStatus::Warming | PendingCanaryReservationStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingCanaryReservationStatus::Warming => {
                match next {
                    PendingCanaryReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingCanaryReservationStatus::Ready => {
                return next == PendingCanaryReservationStatus::Draining
                    || next == PendingCanaryReservationStatus::Stopped;
            }
            PendingCanaryReservationStatus::Draining => {
                match next {
                    PendingCanaryReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingCanaryReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending canary reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingCanaryReservation {
        PendingCanaryReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            ttl_seconds: self.ttl_seconds,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending canary reservation in status idle.
pub fn new_pending_canary_reservation(code: &str, label: &str) -> PendingCanaryReservation {
    PendingCanaryReservation {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        ttl_seconds: 0,
        cost: 0.0,
        status: PendingCanaryReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_canary_reservation("code-626", "label-532");
        assert_eq!(item.status, PendingCanaryReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_pending_canary_reservation("code-626", "label-532");
        if item.move_to(PendingCanaryReservationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingCanaryReservationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_canary_reservation("code-626", "label-532");
        item.status = PendingCanaryReservationStatus::Stopped;
        assert!(
            item.move_to(PendingCanaryReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
