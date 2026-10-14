use super::*;

use acme_kit::text;

/// Where a segment review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SegmentReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SegmentReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a segment review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReview {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub burst: i64,
    pub title: String,
    pub status: SegmentReviewStatus,
}

impl SegmentReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentReviewStatus) -> bool {
        match self.status {
            SegmentReviewStatus::New => {
                return next == SegmentReviewStatus::Done;
            }
            SegmentReviewStatus::Queued => {
                match next {
                    SegmentReviewStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentReviewStatus::Running => {
                return next == SegmentReviewStatus::Failed;
            }
            SegmentReviewStatus::Done => {
                match next {
                    SegmentReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: SegmentReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> SegmentReview {
        SegmentReview {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            burst: self.burst,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a segment review in status new.
pub fn new_segment_review(code: &str, name: &str) -> SegmentReview {
    SegmentReview {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        burst: 0,
        title: "".to_string(),
        status: SegmentReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_segment_review("code-924", "name-823");
        assert_eq!(item.status, SegmentReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_segment_review("code-924", "name-823");
        if item.move_to(SegmentReviewStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentReviewStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_segment_review("code-924", "name-823");
        item.status = SegmentReviewStatus::Failed;
        assert!(item.move_to(SegmentReviewStatus::New).is_err(), "expected move to fail");
    }
}
