use super::*;

use acme_kit::text;

/// Where a seasonal identity review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalIdentityReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalIdentityReviewStatus {
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

impl std::fmt::Display for SeasonalIdentityReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal identity review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityReview {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub segment: String,
    pub principal: String,
    pub description: String,
    pub status: SeasonalIdentityReviewStatus,
}

impl SeasonalIdentityReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalIdentityReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalIdentityReviewStatus) -> bool {
        match self.status {
            SeasonalIdentityReviewStatus::Idle => {
                return next == SeasonalIdentityReviewStatus::Warming || next == SeasonalIdentityReviewStatus::Ready;
            }
            SeasonalIdentityReviewStatus::Warming => {
                match next {
                    SeasonalIdentityReviewStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIdentityReviewStatus::Ready => {
                return next == SeasonalIdentityReviewStatus::Stopped;
            }
            SeasonalIdentityReviewStatus::Draining => {
                return next == SeasonalIdentityReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalIdentityReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal identity review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> SeasonalIdentityReview {
        SeasonalIdentityReview {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            segment: self.segment.clone(),
            principal: self.principal.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a seasonal identity review in status idle.
pub fn new_seasonal_identity_review(code: &str, title: &str) -> SeasonalIdentityReview {
    SeasonalIdentityReview {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        segment: "".to_string(),
        principal: "".to_string(),
        description: "".to_string(),
        status: SeasonalIdentityReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_identity_review("code-991", "title-295");
        assert_eq!(item.status, SeasonalIdentityReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_seasonal_identity_review("code-991", "title-295");
        if item.move_to(SeasonalIdentityReviewStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalIdentityReviewStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_identity_review("code-991", "title-295");
        item.status = SeasonalIdentityReviewStatus::Stopped;
        assert!(
            item.move_to(SeasonalIdentityReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
