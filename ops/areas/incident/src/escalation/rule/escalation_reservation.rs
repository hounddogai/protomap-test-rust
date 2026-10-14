use super::*;

use acme_kit::text;

/// Where a escalation reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EscalationReservationStatus {
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

impl std::fmt::Display for EscalationReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationReservation {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub created_at: i64,
    pub maximum: f64,
    pub threshold: f64,
    pub status: EscalationReservationStatus,
}

impl EscalationReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationReservationStatus) -> bool {
        match self.status {
            EscalationReservationStatus::Idle => {
                return next == EscalationReservationStatus::Stopped;
            }
            EscalationReservationStatus::Warming => {
                match next {
                    EscalationReservationStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationReservationStatus::Ready => {
                match next {
                    EscalationReservationStatus::Draining | EscalationReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationReservationStatus::Draining => {
                match next {
                    EscalationReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EscalationReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation reservation", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.retries as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationReservation {
        EscalationReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            created_at: self.created_at,
            maximum: self.maximum,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a escalation reservation in status idle.
pub fn new_escalation_reservation(code: &str, label: &str) -> EscalationReservation {
    EscalationReservation {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        created_at: 0,
        maximum: 0.0,
        threshold: 0.0,
        status: EscalationReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_escalation_reservation("code-508", "label-278");
        assert_eq!(item.status, EscalationReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_escalation_reservation("code-508", "label-278");
        if item.move_to(EscalationReservationStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationReservationStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_escalation_reservation("code-508", "label-278");
        item.status = EscalationReservationStatus::Stopped;
        assert!(
            item.move_to(EscalationReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
