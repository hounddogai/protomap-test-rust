use super::*;

use acme_kit::text;

/// Where a batched freeze review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFreezeReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedFreezeReviewStatus {
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

impl std::fmt::Display for BatchedFreezeReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched freeze review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFreezeReview {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub attempts: i64,
    pub port: i64,
    pub pinned: bool,
    pub capacity: i64,
    pub status: BatchedFreezeReviewStatus,
}

impl BatchedFreezeReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFreezeReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFreezeReviewStatus) -> bool {
        match self.status {
            BatchedFreezeReviewStatus::Idle => {
                return next == BatchedFreezeReviewStatus::Warming;
            }
            BatchedFreezeReviewStatus::Warming => {
                return next == BatchedFreezeReviewStatus::Ready;
            }
            BatchedFreezeReviewStatus::Ready => {
                match next {
                    BatchedFreezeReviewStatus::Draining | BatchedFreezeReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedFreezeReviewStatus::Draining => {
                match next {
                    BatchedFreezeReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedFreezeReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched freeze review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.capacity as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedFreezeReview {
        BatchedFreezeReview {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            attempts: self.attempts,
            port: self.port,
            pinned: self.pinned,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched freeze review in status idle.
pub fn new_batched_freeze_review(code: &str, title: &str) -> BatchedFreezeReview {
    BatchedFreezeReview {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        attempts: 0,
        port: 0,
        pinned: false,
        capacity: 0,
        status: BatchedFreezeReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_freeze_review("code-282", "title-800");
        assert_eq!(item.status, BatchedFreezeReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_freeze_review("code-282", "title-800");
        if item.move_to(BatchedFreezeReviewStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedFreezeReviewStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_freeze_review("code-282", "title-800");
        item.status = BatchedFreezeReviewStatus::Stopped;
        assert!(
            item.move_to(BatchedFreezeReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
