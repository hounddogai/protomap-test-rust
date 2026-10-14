use super::*;

use acme_kit::text;

/// Where a batched release reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedReleaseReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedReleaseReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedReleaseReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched release reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseReservation {
    pub code: String,
    pub name: String,
    pub country: String,
    pub cost: f64,
    pub description: String,
    pub status: BatchedReleaseReservationStatus,
}

impl BatchedReleaseReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedReleaseReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedReleaseReservationStatus) -> bool {
        match self.status {
            BatchedReleaseReservationStatus::Requested => {
                return next == BatchedReleaseReservationStatus::Expired;
            }
            BatchedReleaseReservationStatus::Granted => {
                match next {
                    BatchedReleaseReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedReleaseReservationStatus::Revoked => {
                match next {
                    BatchedReleaseReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: BatchedReleaseReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched release reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> BatchedReleaseReservation {
        BatchedReleaseReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            cost: self.cost,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched release reservation in status requested.
pub fn new_batched_release_reservation(code: &str, name: &str) -> BatchedReleaseReservation {
    BatchedReleaseReservation {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        cost: 0.0,
        description: "".to_string(),
        status: BatchedReleaseReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_release_reservation("code-297", "name-265");
        assert_eq!(item.status, BatchedReleaseReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_batched_release_reservation("code-297", "name-265");
        if item.move_to(BatchedReleaseReservationStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedReleaseReservationStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_release_reservation("code-297", "name-265");
        item.status = BatchedReleaseReservationStatus::Expired;
        assert!(
            item.move_to(BatchedReleaseReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
