use super::*;

use acme_kit::text;

/// Where a batched timeline record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTimelineRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedTimelineRecordStatus {
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

impl std::fmt::Display for BatchedTimelineRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched timeline record as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecord {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub grace_days: i64,
    pub priority: i64,
    pub floor_value: f64,
    pub budget: f64,
    pub sequence: i64,
    pub status: BatchedTimelineRecordStatus,
}

impl BatchedTimelineRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTimelineRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTimelineRecordStatus) -> bool {
        match self.status {
            BatchedTimelineRecordStatus::New => {
                return next == BatchedTimelineRecordStatus::Queued;
            }
            BatchedTimelineRecordStatus::Queued => {
                return next == BatchedTimelineRecordStatus::Running || next == BatchedTimelineRecordStatus::Done;
            }
            BatchedTimelineRecordStatus::Running => {
                match next {
                    BatchedTimelineRecordStatus::Done | BatchedTimelineRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTimelineRecordStatus::Done => {
                return next == BatchedTimelineRecordStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTimelineRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched timeline record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTimelineRecord {
        BatchedTimelineRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            grace_days: self.grace_days,
            priority: self.priority,
            floor_value: self.floor_value,
            budget: self.budget,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched timeline record in status new.
pub fn new_batched_timeline_record(code: &str, name: &str) -> BatchedTimelineRecord {
    BatchedTimelineRecord {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        grace_days: 0,
        priority: 0,
        floor_value: 0.0,
        budget: 0.0,
        sequence: 0,
        status: BatchedTimelineRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_timeline_record("code-43", "name-457");
        assert_eq!(item.status, BatchedTimelineRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_timeline_record("code-43", "name-457");
        if item.move_to(BatchedTimelineRecordStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTimelineRecordStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_timeline_record("code-43", "name-457");
        item.status = BatchedTimelineRecordStatus::Failed;
        assert!(
            item.move_to(BatchedTimelineRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
