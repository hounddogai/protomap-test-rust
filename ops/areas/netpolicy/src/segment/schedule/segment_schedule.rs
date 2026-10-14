use super::*;

use acme_kit::text;

/// Where a segment schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SegmentScheduleStatus {
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

impl std::fmt::Display for SegmentScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a segment schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSchedule {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub pinned: bool,
    pub status: SegmentScheduleStatus,
}

impl SegmentSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentScheduleStatus) -> bool {
        match self.status {
            SegmentScheduleStatus::New => {
                return next == SegmentScheduleStatus::Queued || next == SegmentScheduleStatus::Failed;
            }
            SegmentScheduleStatus::Queued => {
                return next == SegmentScheduleStatus::Done;
            }
            SegmentScheduleStatus::Running => {
                return next == SegmentScheduleStatus::Failed;
            }
            SegmentScheduleStatus::Done => {
                return next == SegmentScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "segment schedule", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> SegmentSchedule {
        SegmentSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment schedule in status new.
pub fn new_segment_schedule(code: &str, title: &str) -> SegmentSchedule {
    SegmentSchedule {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        ttl_seconds: 0,
        owner_id: "".to_string(),
        pinned: false,
        status: SegmentScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_segment_schedule("code-892", "title-387");
        assert_eq!(item.status, SegmentScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_segment_schedule("code-892", "title-387");
        if item.move_to(SegmentScheduleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentScheduleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_segment_schedule("code-892", "title-387");
        item.status = SegmentScheduleStatus::Failed;
        assert!(
            item.move_to(SegmentScheduleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
