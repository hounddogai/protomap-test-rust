use super::*;

use acme_kit::text;

/// Where a batched silence segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSilenceSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedSilenceSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedSilenceSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched silence segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSilenceSegment {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub weight: f64,
    pub duration_minutes: i64,
    pub attempts: i64,
    pub status: BatchedSilenceSegmentStatus,
}

impl BatchedSilenceSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSilenceSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSilenceSegmentStatus) -> bool {
        match self.status {
            BatchedSilenceSegmentStatus::Draft => {
                return next == BatchedSilenceSegmentStatus::Suspended || next == BatchedSilenceSegmentStatus::Closed;
            }
            BatchedSilenceSegmentStatus::Pending => {
                match next {
                    BatchedSilenceSegmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSilenceSegmentStatus::Active => {
                return next == BatchedSilenceSegmentStatus::Suspended || next == BatchedSilenceSegmentStatus::Closed;
            }
            BatchedSilenceSegmentStatus::Suspended => {
                return next == BatchedSilenceSegmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSilenceSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched silence segment {} cannot move from {} to {}",
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
        ((self.cost + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSilenceSegment {
        BatchedSilenceSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            weight: self.weight,
            duration_minutes: self.duration_minutes,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched silence segment in status draft.
pub fn new_batched_silence_segment(code: &str, title: &str) -> BatchedSilenceSegment {
    BatchedSilenceSegment {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        weight: 0.0,
        duration_minutes: 0,
        attempts: 0,
        status: BatchedSilenceSegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_silence_segment("code-51", "title-587");
        assert_eq!(item.status, BatchedSilenceSegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_batched_silence_segment("code-51", "title-587");
        if item.move_to(BatchedSilenceSegmentStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSilenceSegmentStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_silence_segment("code-51", "title-587");
        item.status = BatchedSilenceSegmentStatus::Closed;
        assert!(
            item.move_to(BatchedSilenceSegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
