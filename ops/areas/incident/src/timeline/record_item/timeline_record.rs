use super::*;

use acme_kit::text;

/// Where a timeline record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineRecordStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TimelineRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRecord {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub summary: String,
    pub locale: String,
    pub minimum: f64,
    pub status: TimelineRecordStatus,
}

impl TimelineRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineRecordStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineRecordStatus) -> bool {
        match self.status {
            TimelineRecordStatus::Planned => {
                match next {
                    TimelineRecordStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineRecordStatus::Scheduled => {
                return next == TimelineRecordStatus::InProgress || next == TimelineRecordStatus::Completed;
            }
            TimelineRecordStatus::InProgress => {
                match next {
                    TimelineRecordStatus::Completed | TimelineRecordStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineRecordStatus::Completed => {
                match next {
                    TimelineRecordStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TimelineRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "timeline record", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.minimum + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineRecord {
        TimelineRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            summary: self.summary.clone(),
            locale: self.locale.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a timeline record in status planned.
pub fn new_timeline_record(code: &str, name: &str) -> TimelineRecord {
    TimelineRecord {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        summary: "".to_string(),
        locale: "".to_string(),
        minimum: 0.0,
        status: TimelineRecordStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_timeline_record("code-604", "name-979");
        assert_eq!(item.status, TimelineRecordStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_timeline_record("code-604", "name-979");
        if item.move_to(TimelineRecordStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineRecordStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_timeline_record("code-604", "name-979");
        item.status = TimelineRecordStatus::Cancelled;
        assert!(
            item.move_to(TimelineRecordStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
