use super::*;

use acme_kit::text;

/// Where a global segment schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSegmentScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalSegmentScheduleStatus {
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

impl std::fmt::Display for GlobalSegmentScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global segment schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentSchedule {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub timezone: String,
    pub archived: bool,
    pub scope: String,
    pub external_ref: String,
    pub sequence: i64,
    pub grace_days: i64,
    pub duration_minutes: i64,
    pub status: GlobalSegmentScheduleStatus,
}

impl GlobalSegmentSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSegmentScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSegmentScheduleStatus) -> bool {
        match self.status {
            GlobalSegmentScheduleStatus::New => {
                match next {
                    GlobalSegmentScheduleStatus::Running | GlobalSegmentScheduleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentScheduleStatus::Queued => {
                match next {
                    GlobalSegmentScheduleStatus::Running | GlobalSegmentScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentScheduleStatus::Running => {
                return next == GlobalSegmentScheduleStatus::Done || next == GlobalSegmentScheduleStatus::Failed;
            }
            GlobalSegmentScheduleStatus::Done => {
                return next == GlobalSegmentScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSegmentScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global segment schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global segment schedule", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSegmentSchedule {
        GlobalSegmentSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            timezone: self.timezone.clone(),
            archived: self.archived,
            scope: self.scope.clone(),
            external_ref: self.external_ref.clone(),
            sequence: self.sequence,
            grace_days: self.grace_days,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a global segment schedule in status new.
pub fn new_global_segment_schedule(code: &str, label: &str) -> GlobalSegmentSchedule {
    GlobalSegmentSchedule {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        timezone: "".to_string(),
        archived: false,
        scope: "".to_string(),
        external_ref: "".to_string(),
        sequence: 0,
        grace_days: 0,
        duration_minutes: 0,
        status: GlobalSegmentScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_segment_schedule("code-32", "label-305");
        assert_eq!(item.status, GlobalSegmentScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_global_segment_schedule("code-32", "label-305");
        if item.move_to(GlobalSegmentScheduleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSegmentScheduleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_segment_schedule("code-32", "label-305");
        item.status = GlobalSegmentScheduleStatus::Failed;
        assert!(
            item.move_to(GlobalSegmentScheduleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
