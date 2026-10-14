use super::*;

use acme_kit::text;

/// Where a reservation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReservationClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReservationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationClaim {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub retries: i64,
    pub window_days: i64,
    pub minimum: f64,
    pub status: ReservationClaimStatus,
}

impl ReservationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationClaimStatus) -> bool {
        match self.status {
            ReservationClaimStatus::Proposed => {
                match next {
                    ReservationClaimStatus::Approved | ReservationClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationClaimStatus::Approved => {
                return next == ReservationClaimStatus::Rejected;
            }
            ReservationClaimStatus::Rejected => {
                match next {
                    ReservationClaimStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ReservationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.minimum + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationClaim {
        ReservationClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            retries: self.retries,
            window_days: self.window_days,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a reservation claim in status proposed.
pub fn new_reservation_claim(code: &str, label: &str) -> ReservationClaim {
    ReservationClaim {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        retries: 0,
        window_days: 0,
        minimum: 0.0,
        status: ReservationClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_reservation_claim("code-342", "label-767");
        assert_eq!(item.status, ReservationClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_reservation_claim("code-342", "label-767");
        if item.move_to(ReservationClaimStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationClaimStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_reservation_claim("code-342", "label-767");
        item.status = ReservationClaimStatus::Withdrawn;
        assert!(
            item.move_to(ReservationClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
