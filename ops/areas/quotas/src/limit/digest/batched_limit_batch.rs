use super::*;

use acme_kit::text;

/// Where a batched limit batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedLimitBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedLimitBatchStatus {
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

impl std::fmt::Display for BatchedLimitBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched limit batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitBatch {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub floor_value: f64,
    pub duration_minutes: i64,
    pub batch_size: i64,
    pub margin: f64,
    pub archived: bool,
    pub status: BatchedLimitBatchStatus,
}

impl BatchedLimitBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedLimitBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedLimitBatchStatus) -> bool {
        match self.status {
            BatchedLimitBatchStatus::Idle => {
                return next == BatchedLimitBatchStatus::Warming;
            }
            BatchedLimitBatchStatus::Warming => {
                match next {
                    BatchedLimitBatchStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedLimitBatchStatus::Ready => {
                return next == BatchedLimitBatchStatus::Stopped;
            }
            BatchedLimitBatchStatus::Draining => {
                match next {
                    BatchedLimitBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedLimitBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched limit batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.batch_size as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedLimitBatch {
        BatchedLimitBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            floor_value: self.floor_value,
            duration_minutes: self.duration_minutes,
            batch_size: self.batch_size,
            margin: self.margin,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched limit batch in status idle.
pub fn new_batched_limit_batch(code: &str, title: &str) -> BatchedLimitBatch {
    BatchedLimitBatch {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        floor_value: 0.0,
        duration_minutes: 0,
        batch_size: 0,
        margin: 0.0,
        archived: false,
        status: BatchedLimitBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_limit_batch("code-156", "title-976");
        assert_eq!(item.status, BatchedLimitBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_limit_batch("code-156", "title-976");
        if item.move_to(BatchedLimitBatchStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedLimitBatchStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_limit_batch("code-156", "title-976");
        item.status = BatchedLimitBatchStatus::Stopped;
        assert!(
            item.move_to(BatchedLimitBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
