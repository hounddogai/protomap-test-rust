use super::*;

use acme_kit::text;

/// Where a timeline cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TimelineCycleStatus {
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

impl std::fmt::Display for TimelineCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCycle {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub ttl_seconds: i64,
    pub window_days: i64,
    pub checksum: String,
    pub threshold: f64,
    pub discount: f64,
    pub status: TimelineCycleStatus,
}

impl TimelineCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineCycleStatus) -> bool {
        match self.status {
            TimelineCycleStatus::New => {
                match next {
                    TimelineCycleStatus::Running | TimelineCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineCycleStatus::Queued => {
                match next {
                    TimelineCycleStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineCycleStatus::Running => {
                return next == TimelineCycleStatus::Failed;
            }
            TimelineCycleStatus::Done => {
                match next {
                    TimelineCycleStatus::Failed => {
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

    pub fn move_to(&mut self, next: TimelineCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.window_days as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineCycle {
        TimelineCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            ttl_seconds: self.ttl_seconds,
            window_days: self.window_days,
            checksum: self.checksum.clone(),
            threshold: self.threshold,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a timeline cycle in status new.
pub fn new_timeline_cycle(code: &str, name: &str) -> TimelineCycle {
    TimelineCycle {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        ttl_seconds: 0,
        window_days: 0,
        checksum: "".to_string(),
        threshold: 0.0,
        discount: 0.0,
        status: TimelineCycleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_timeline_cycle("code-178", "name-935");
        assert_eq!(item.status, TimelineCycleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_timeline_cycle("code-178", "name-935");
        if item.move_to(TimelineCycleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineCycleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_timeline_cycle("code-178", "name-935");
        item.status = TimelineCycleStatus::Failed;
        assert!(item.move_to(TimelineCycleStatus::New).is_err(), "expected move to fail");
    }
}
