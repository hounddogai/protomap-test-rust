use super::*;

use acme_kit::text;

/// Where a timeline profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TimelineProfileStatus {
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

impl std::fmt::Display for TimelineProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline profile as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProfile {
    pub code: String,
    pub name: String,
    pub region: String,
    pub rotation_days: i64,
    pub currency: String,
    pub floor_value: f64,
    pub duration_minutes: i64,
    pub status: TimelineProfileStatus,
}

impl TimelineProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineProfileStatus) -> bool {
        match self.status {
            TimelineProfileStatus::New => {
                match next {
                    TimelineProfileStatus::Running | TimelineProfileStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineProfileStatus::Queued => {
                return next == TimelineProfileStatus::Done || next == TimelineProfileStatus::Failed;
            }
            TimelineProfileStatus::Running => {
                match next {
                    TimelineProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineProfileStatus::Done => {
                match next {
                    TimelineProfileStatus::Failed => {
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

    pub fn move_to(&mut self, next: TimelineProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.rotation_days as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineProfile {
        TimelineProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            rotation_days: self.rotation_days,
            currency: self.currency.clone(),
            floor_value: self.floor_value,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a timeline profile in status new.
pub fn new_timeline_profile(code: &str, name: &str) -> TimelineProfile {
    TimelineProfile {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        rotation_days: 0,
        currency: "".to_string(),
        floor_value: 0.0,
        duration_minutes: 0,
        status: TimelineProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_timeline_profile("code-832", "name-761");
        assert_eq!(item.status, TimelineProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_timeline_profile("code-832", "name-761");
        if item.move_to(TimelineProfileStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineProfileStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_timeline_profile("code-832", "name-761");
        item.status = TimelineProfileStatus::Failed;
        assert!(
            item.move_to(TimelineProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
