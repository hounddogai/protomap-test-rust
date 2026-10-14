use super::*;

use acme_kit::text;

/// Where a strict freeze review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFreezeReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictFreezeReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictFreezeReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict freeze review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReview {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub margin: f64,
    pub pinned: bool,
    pub archived: bool,
    pub rate: f64,
    pub contact_email: String,
    pub status: StrictFreezeReviewStatus,
}

impl StrictFreezeReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFreezeReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFreezeReviewStatus) -> bool {
        match self.status {
            StrictFreezeReviewStatus::Requested => {
                match next {
                    StrictFreezeReviewStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFreezeReviewStatus::Granted => {
                return next == StrictFreezeReviewStatus::Revoked;
            }
            StrictFreezeReviewStatus::Revoked => {
                return next == StrictFreezeReviewStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictFreezeReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict freeze review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.margin + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFreezeReview {
        StrictFreezeReview {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            margin: self.margin,
            pinned: self.pinned,
            archived: self.archived,
            rate: self.rate,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict freeze review in status requested.
pub fn new_strict_freeze_review(code: &str, name: &str) -> StrictFreezeReview {
    StrictFreezeReview {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        margin: 0.0,
        pinned: false,
        archived: false,
        rate: 0.0,
        contact_email: "".to_string(),
        status: StrictFreezeReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_freeze_review("code-642", "name-122");
        assert_eq!(item.status, StrictFreezeReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_strict_freeze_review("code-642", "name-122");
        if item.move_to(StrictFreezeReviewStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFreezeReviewStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_freeze_review("code-642", "name-122");
        item.status = StrictFreezeReviewStatus::Expired;
        assert!(
            item.move_to(StrictFreezeReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
