use super::*;

use acme_kit::text;

/// Where a sink schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SinkScheduleStatus {
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

impl std::fmt::Display for SinkScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink schedule as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkSchedule {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub reviewer: String,
    pub pinned: bool,
    pub score: f64,
    pub grace_days: i64,
    pub segment: String,
    pub quota_units: i64,
    pub scope: String,
    pub budget: f64,
    pub status: SinkScheduleStatus,
}

impl SinkSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkScheduleStatus) -> bool {
        match self.status {
            SinkScheduleStatus::New => {
                match next {
                    SinkScheduleStatus::Queued | SinkScheduleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkScheduleStatus::Queued => {
                match next {
                    SinkScheduleStatus::Running | SinkScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkScheduleStatus::Running => {
                match next {
                    SinkScheduleStatus::Done | SinkScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkScheduleStatus::Done => {
                return next == SinkScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink schedule", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.grace_days as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkSchedule {
        SinkSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            reviewer: self.reviewer.clone(),
            pinned: self.pinned,
            score: self.score,
            grace_days: self.grace_days,
            segment: self.segment.clone(),
            quota_units: self.quota_units,
            scope: self.scope.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a sink schedule in status new.
pub fn new_sink_schedule(code: &str, title: &str) -> SinkSchedule {
    SinkSchedule {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        reviewer: "".to_string(),
        pinned: false,
        score: 0.0,
        grace_days: 0,
        segment: "".to_string(),
        quota_units: 0,
        scope: "".to_string(),
        budget: 0.0,
        status: SinkScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_sink_schedule("code-34", "title-500");
        assert_eq!(item.status, SinkScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_sink_schedule("code-34", "title-500");
        if item.move_to(SinkScheduleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkScheduleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_sink_schedule("code-34", "title-500");
        item.status = SinkScheduleStatus::Failed;
        assert!(item.move_to(SinkScheduleStatus::New).is_err(), "expected move to fail");
    }
}
