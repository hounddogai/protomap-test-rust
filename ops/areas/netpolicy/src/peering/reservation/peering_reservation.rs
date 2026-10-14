use super::*;

use acme_kit::text;

/// Where a peering reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PeeringReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PeeringReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReservation {
    pub code: String,
    pub name: String,
    pub description: String,
    pub ends_at: i64,
    pub audience: String,
    pub status: PeeringReservationStatus,
}

impl PeeringReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringReservationStatus) -> bool {
        match self.status {
            PeeringReservationStatus::Open => {
                return next == PeeringReservationStatus::Resolved || next == PeeringReservationStatus::Reopened;
            }
            PeeringReservationStatus::InReview => {
                match next {
                    PeeringReservationStatus::Reopened | PeeringReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringReservationStatus::Resolved => {
                match next {
                    PeeringReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringReservationStatus::Reopened => {
                return next == PeeringReservationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> PeeringReservation {
        PeeringReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            ends_at: self.ends_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a peering reservation in status open.
pub fn new_peering_reservation(code: &str, name: &str) -> PeeringReservation {
    PeeringReservation {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        ends_at: 0,
        audience: "".to_string(),
        status: PeeringReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_peering_reservation("code-362", "name-587");
        assert_eq!(item.status, PeeringReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_peering_reservation("code-362", "name-587");
        if item.move_to(PeeringReservationStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringReservationStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_peering_reservation("code-362", "name-587");
        item.status = PeeringReservationStatus::Archived;
        assert!(
            item.move_to(PeeringReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
