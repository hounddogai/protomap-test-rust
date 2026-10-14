use super::*;

use acme_kit::text;

/// Where a egress reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EgressReservationStatus {
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

impl std::fmt::Display for EgressReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress reservation as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressReservation {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub limit: i64,
    pub retries: i64,
    pub sequence: i64,
    pub status: EgressReservationStatus,
}

impl EgressReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressReservationStatus) -> bool {
        match self.status {
            EgressReservationStatus::Idle => {
                return next == EgressReservationStatus::Ready;
            }
            EgressReservationStatus::Warming => {
                match next {
                    EgressReservationStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressReservationStatus::Ready => {
                return next == EgressReservationStatus::Draining;
            }
            EgressReservationStatus::Draining => {
                match next {
                    EgressReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EgressReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "egress reservation", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.sequence as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressReservation {
        EgressReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            limit: self.limit,
            retries: self.retries,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a egress reservation in status idle.
pub fn new_egress_reservation(code: &str, label: &str) -> EgressReservation {
    EgressReservation {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        limit: 0,
        retries: 0,
        sequence: 0,
        status: EgressReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_egress_reservation("code-563", "label-221");
        assert_eq!(item.status, EgressReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_egress_reservation("code-563", "label-221");
        if item.move_to(EgressReservationStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressReservationStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_egress_reservation("code-563", "label-221");
        item.status = EgressReservationStatus::Stopped;
        assert!(
            item.move_to(EgressReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
