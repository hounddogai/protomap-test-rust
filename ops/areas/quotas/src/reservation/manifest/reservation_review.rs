use super::*;

use acme_kit::text;

/// Where a reservation review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReservationReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReservationReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationReview {
    pub code: String,
    pub name: String,
    pub region: String,
    pub discount: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub owner_id: String,
    pub score: f64,
    pub burst: i64,
    pub fingerprint: String,
    pub priority: i64,
    pub status: ReservationReviewStatus,
}

impl ReservationReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationReviewStatus) -> bool {
        match self.status {
            ReservationReviewStatus::Proposed => {
                return next == ReservationReviewStatus::Approved || next == ReservationReviewStatus::Withdrawn;
            }
            ReservationReviewStatus::Approved => {
                return next == ReservationReviewStatus::Rejected;
            }
            ReservationReviewStatus::Rejected => {
                return next == ReservationReviewStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation review {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "reservation review", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.discount * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationReview {
        ReservationReview {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            discount: self.discount,
            created_at: self.created_at,
            updated_at: self.updated_at,
            owner_id: self.owner_id.clone(),
            score: self.score,
            burst: self.burst,
            fingerprint: self.fingerprint.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a reservation review in status proposed.
pub fn new_reservation_review(code: &str, name: &str) -> ReservationReview {
    ReservationReview {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        discount: 0.0,
        created_at: 0,
        updated_at: 0,
        owner_id: "".to_string(),
        score: 0.0,
        burst: 0,
        fingerprint: "".to_string(),
        priority: 0,
        status: ReservationReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_reservation_review("code-324", "name-448");
        assert_eq!(item.status, ReservationReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_reservation_review("code-324", "name-448");
        if item.move_to(ReservationReviewStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationReviewStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_reservation_review("code-324", "name-448");
        item.status = ReservationReviewStatus::Withdrawn;
        assert!(
            item.move_to(ReservationReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
