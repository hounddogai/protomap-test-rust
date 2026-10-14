use super::*;

use acme_kit::text;

/// Where a timeline review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TimelineReviewStatus {
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

impl std::fmt::Display for TimelineReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReview {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub currency: String,
    pub channel: String,
    pub visible: bool,
    pub unit: String,
    pub quantity: i64,
    pub status: TimelineReviewStatus,
}

impl TimelineReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineReviewStatus) -> bool {
        match self.status {
            TimelineReviewStatus::Open => {
                match next {
                    TimelineReviewStatus::Resolved | TimelineReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineReviewStatus::InReview => {
                match next {
                    TimelineReviewStatus::Reopened | TimelineReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineReviewStatus::Resolved => {
                match next {
                    TimelineReviewStatus::Reopened | TimelineReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineReviewStatus::Reopened => {
                match next {
                    TimelineReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: TimelineReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "timeline review", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> TimelineReview {
        TimelineReview {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            currency: self.currency.clone(),
            channel: self.channel.clone(),
            visible: self.visible,
            unit: self.unit.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a timeline review in status open.
pub fn new_timeline_review(code: &str, title: &str) -> TimelineReview {
    TimelineReview {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        currency: "".to_string(),
        channel: "".to_string(),
        visible: false,
        unit: "".to_string(),
        quantity: 0,
        status: TimelineReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_timeline_review("code-987", "title-227");
        assert_eq!(item.status, TimelineReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_timeline_review("code-987", "title-227");
        if item.move_to(TimelineReviewStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineReviewStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_timeline_review("code-987", "title-227");
        item.status = TimelineReviewStatus::Archived;
        assert!(
            item.move_to(TimelineReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
