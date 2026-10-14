use super::*;

use acme_kit::text;

/// Where a regional reservation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalReservationClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalReservationClaimStatus {
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

impl std::fmt::Display for RegionalReservationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional reservation claim as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationClaim {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub cost: f64,
    pub audience: String,
    pub batch_size: i64,
    pub amount: f64,
    pub window_days: i64,
    pub quantity: i64,
    pub rotation_days: i64,
    pub status: RegionalReservationClaimStatus,
}

impl RegionalReservationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalReservationClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalReservationClaimStatus) -> bool {
        match self.status {
            RegionalReservationClaimStatus::Open => {
                return next == RegionalReservationClaimStatus::Reopened
                    || next == RegionalReservationClaimStatus::Archived;
            }
            RegionalReservationClaimStatus::InReview => {
                return next == RegionalReservationClaimStatus::Resolved
                    || next == RegionalReservationClaimStatus::Reopened;
            }
            RegionalReservationClaimStatus::Resolved => {
                return next == RegionalReservationClaimStatus::Reopened
                    || next == RegionalReservationClaimStatus::Archived;
            }
            RegionalReservationClaimStatus::Reopened => {
                return next == RegionalReservationClaimStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalReservationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional reservation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional reservation claim", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.amount * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalReservationClaim {
        RegionalReservationClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            cost: self.cost,
            audience: self.audience.clone(),
            batch_size: self.batch_size,
            amount: self.amount,
            window_days: self.window_days,
            quantity: self.quantity,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional reservation claim in status open.
pub fn new_regional_reservation_claim(code: &str, title: &str) -> RegionalReservationClaim {
    RegionalReservationClaim {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        cost: 0.0,
        audience: "".to_string(),
        batch_size: 0,
        amount: 0.0,
        window_days: 0,
        quantity: 0,
        rotation_days: 0,
        status: RegionalReservationClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_reservation_claim("code-173", "title-244");
        assert_eq!(item.status, RegionalReservationClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_regional_reservation_claim("code-173", "title-244");
        if item.move_to(RegionalReservationClaimStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalReservationClaimStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_reservation_claim("code-173", "title-244");
        item.status = RegionalReservationClaimStatus::Archived;
        assert!(
            item.move_to(RegionalReservationClaimStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
