use super::*;

use acme_kit::text;

/// Where a batched escalation reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedEscalationReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedEscalationReservationStatus {
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

impl std::fmt::Display for BatchedEscalationReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched escalation reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationReservation {
    pub code: String,
    pub name: String,
    pub host: String,
    pub region: String,
    pub approved: bool,
    pub score: f64,
    pub status: BatchedEscalationReservationStatus,
}

impl BatchedEscalationReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedEscalationReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedEscalationReservationStatus) -> bool {
        match self.status {
            BatchedEscalationReservationStatus::Idle => {
                match next {
                    BatchedEscalationReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedEscalationReservationStatus::Warming => {
                return next == BatchedEscalationReservationStatus::Draining
                    || next == BatchedEscalationReservationStatus::Stopped;
            }
            BatchedEscalationReservationStatus::Ready => {
                return next == BatchedEscalationReservationStatus::Draining
                    || next == BatchedEscalationReservationStatus::Stopped;
            }
            BatchedEscalationReservationStatus::Draining => {
                match next {
                    BatchedEscalationReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedEscalationReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched escalation reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> BatchedEscalationReservation {
        BatchedEscalationReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            region: self.region.clone(),
            approved: self.approved,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a batched escalation reservation in status idle.
pub fn new_batched_escalation_reservation(code: &str, name: &str) -> BatchedEscalationReservation {
    BatchedEscalationReservation {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        region: "".to_string(),
        approved: false,
        score: 0.0,
        status: BatchedEscalationReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_escalation_reservation("code-924", "name-938");
        assert_eq!(item.status, BatchedEscalationReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_batched_escalation_reservation("code-924", "name-938");
        if item.move_to(BatchedEscalationReservationStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedEscalationReservationStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_escalation_reservation("code-924", "name-938");
        item.status = BatchedEscalationReservationStatus::Stopped;
        assert!(
            item.move_to(BatchedEscalationReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
