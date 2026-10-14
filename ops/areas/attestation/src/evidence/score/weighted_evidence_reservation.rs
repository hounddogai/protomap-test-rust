use super::*;

use acme_kit::text;

/// Where a weighted evidence reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedEvidenceReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedEvidenceReservationStatus {
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

impl std::fmt::Display for WeightedEvidenceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted evidence reservation as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEvidenceReservation {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub external_ref: String,
    pub retries: i64,
    pub contact_email: String,
    pub status: WeightedEvidenceReservationStatus,
}

impl WeightedEvidenceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedEvidenceReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedEvidenceReservationStatus) -> bool {
        match self.status {
            WeightedEvidenceReservationStatus::Idle => {
                match next {
                    WeightedEvidenceReservationStatus::Ready | WeightedEvidenceReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedEvidenceReservationStatus::Warming => {
                return next == WeightedEvidenceReservationStatus::Ready
                    || next == WeightedEvidenceReservationStatus::Draining;
            }
            WeightedEvidenceReservationStatus::Ready => {
                return next == WeightedEvidenceReservationStatus::Draining;
            }
            WeightedEvidenceReservationStatus::Draining => {
                return next == WeightedEvidenceReservationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedEvidenceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted evidence reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted evidence reservation", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.retries as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedEvidenceReservation {
        WeightedEvidenceReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted evidence reservation in status idle.
pub fn new_weighted_evidence_reservation(code: &str, name: &str) -> WeightedEvidenceReservation {
    WeightedEvidenceReservation {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        external_ref: "".to_string(),
        retries: 0,
        contact_email: "".to_string(),
        status: WeightedEvidenceReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_evidence_reservation("code-243", "name-442");
        assert_eq!(item.status, WeightedEvidenceReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_weighted_evidence_reservation("code-243", "name-442");
        if item.move_to(WeightedEvidenceReservationStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedEvidenceReservationStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_evidence_reservation("code-243", "name-442");
        item.status = WeightedEvidenceReservationStatus::Stopped;
        assert!(
            item.move_to(WeightedEvidenceReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
