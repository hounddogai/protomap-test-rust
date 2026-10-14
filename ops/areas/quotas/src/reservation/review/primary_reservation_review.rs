use super::*;

use acme_kit::text;

/// Where a primary reservation review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReservationReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryReservationReviewStatus {
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

impl std::fmt::Display for PrimaryReservationReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary reservation review as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationReview {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub visible: bool,
    pub title: String,
    pub updated_at: i64,
    pub scope: String,
    pub grace_days: i64,
    pub archived: bool,
    pub status: PrimaryReservationReviewStatus,
}

impl PrimaryReservationReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReservationReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReservationReviewStatus) -> bool {
        match self.status {
            PrimaryReservationReviewStatus::Idle => {
                return next == PrimaryReservationReviewStatus::Ready
                    || next == PrimaryReservationReviewStatus::Stopped;
            }
            PrimaryReservationReviewStatus::Warming => {
                return next == PrimaryReservationReviewStatus::Ready
                    || next == PrimaryReservationReviewStatus::Draining;
            }
            PrimaryReservationReviewStatus::Ready => {
                match next {
                    PrimaryReservationReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationReviewStatus::Draining => {
                return next == PrimaryReservationReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryReservationReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary reservation review {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.grace_days as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryReservationReview {
        PrimaryReservationReview {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            visible: self.visible,
            title: self.title.clone(),
            updated_at: self.updated_at,
            scope: self.scope.clone(),
            grace_days: self.grace_days,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a primary reservation review in status idle.
pub fn new_primary_reservation_review(code: &str, name: &str) -> PrimaryReservationReview {
    PrimaryReservationReview {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        visible: false,
        title: "".to_string(),
        updated_at: 0,
        scope: "".to_string(),
        grace_days: 0,
        archived: false,
        status: PrimaryReservationReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_reservation_review("code-289", "name-8");
        assert_eq!(item.status, PrimaryReservationReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_primary_reservation_review("code-289", "name-8");
        if item.move_to(PrimaryReservationReviewStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryReservationReviewStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_reservation_review("code-289", "name-8");
        item.status = PrimaryReservationReviewStatus::Stopped;
        assert!(
            item.move_to(PrimaryReservationReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
