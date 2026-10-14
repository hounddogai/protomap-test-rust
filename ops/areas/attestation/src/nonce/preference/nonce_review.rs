use super::*;

use acme_kit::text;

/// Where a nonce review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl NonceReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for NonceReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce review as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceReview {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub rotation_days: i64,
    pub quota_units: i64,
    pub visible: bool,
    pub status: NonceReviewStatus,
}

impl NonceReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceReviewStatus) -> bool {
        match self.status {
            NonceReviewStatus::Planned => {
                match next {
                    NonceReviewStatus::Scheduled | NonceReviewStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceReviewStatus::Scheduled => {
                match next {
                    NonceReviewStatus::InProgress | NonceReviewStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceReviewStatus::InProgress => {
                return next == NonceReviewStatus::Completed || next == NonceReviewStatus::Cancelled;
            }
            NonceReviewStatus::Completed => {
                match next {
                    NonceReviewStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: NonceReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce review", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rotation_days as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceReview {
        NonceReview {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            rotation_days: self.rotation_days,
            quota_units: self.quota_units,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a nonce review in status planned.
pub fn new_nonce_review(code: &str, name: &str) -> NonceReview {
    NonceReview {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        rotation_days: 0,
        quota_units: 0,
        visible: false,
        status: NonceReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_nonce_review("code-435", "name-385");
        assert_eq!(item.status, NonceReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_nonce_review("code-435", "name-385");
        if item.move_to(NonceReviewStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceReviewStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_nonce_review("code-435", "name-385");
        item.status = NonceReviewStatus::Cancelled;
        assert!(
            item.move_to(NonceReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
