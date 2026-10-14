use super::*;

use acme_kit::text;

/// Where a reservation plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReservationPlanStatus {
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

impl std::fmt::Display for ReservationPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One reservation plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPlan {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub source: String,
    pub fingerprint: String,
    pub minimum: f64,
    pub batch_size: i64,
    pub status: ReservationPlanStatus,
}

impl ReservationPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationPlanStatus) -> bool {
        match self.status {
            ReservationPlanStatus::Open => {
                match next {
                    ReservationPlanStatus::InReview | ReservationPlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationPlanStatus::InReview => {
                return next == ReservationPlanStatus::Resolved || next == ReservationPlanStatus::Archived;
            }
            ReservationPlanStatus::Resolved => {
                match next {
                    ReservationPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationPlanStatus::Reopened => {
                return next == ReservationPlanStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "reservation plan", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.minimum + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationPlan {
        ReservationPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            source: self.source.clone(),
            fingerprint: self.fingerprint.clone(),
            minimum: self.minimum,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a reservation plan in status open.
pub fn new_reservation_plan(code: &str, title: &str) -> ReservationPlan {
    ReservationPlan {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        source: "".to_string(),
        fingerprint: "".to_string(),
        minimum: 0.0,
        batch_size: 0,
        status: ReservationPlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_reservation_plan("code-471", "title-471");
        assert_eq!(item.status, ReservationPlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_reservation_plan("code-471", "title-471");
        if item.move_to(ReservationPlanStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationPlanStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_reservation_plan("code-471", "title-471");
        item.status = ReservationPlanStatus::Archived;
        assert!(
            item.move_to(ReservationPlanStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
