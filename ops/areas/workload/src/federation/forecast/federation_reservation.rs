use super::*;

use acme_kit::text;

/// Where a federation reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FederationReservationStatus {
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

impl std::fmt::Display for FederationReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationReservation {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub phone: String,
    pub batch_size: i64,
    pub status: FederationReservationStatus,
}

impl FederationReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationReservationStatus) -> bool {
        match self.status {
            FederationReservationStatus::Idle => {
                match next {
                    FederationReservationStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationReservationStatus::Warming => {
                return next == FederationReservationStatus::Ready;
            }
            FederationReservationStatus::Ready => {
                return next == FederationReservationStatus::Stopped;
            }
            FederationReservationStatus::Draining => {
                match next {
                    FederationReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FederationReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.margin - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationReservation {
        FederationReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            phone: self.phone.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a federation reservation in status idle.
pub fn new_federation_reservation(code: &str, name: &str) -> FederationReservation {
    FederationReservation {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        phone: "".to_string(),
        batch_size: 0,
        status: FederationReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_federation_reservation("code-675", "name-696");
        assert_eq!(item.status, FederationReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_federation_reservation("code-675", "name-696");
        if item.move_to(FederationReservationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationReservationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_federation_reservation("code-675", "name-696");
        item.status = FederationReservationStatus::Stopped;
        assert!(
            item.move_to(FederationReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
