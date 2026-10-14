use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a failover schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FailoverScheduleStatus {
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

impl std::fmt::Display for FailoverScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSchedule {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub margin: f64,
    pub source: String,
    pub floor_value: f64,
    pub country: String,
    pub rate: f64,
    pub duration_minutes: i64,
    pub status: FailoverScheduleStatus,
}

impl FailoverSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverScheduleStatus) -> bool {
        match self.status {
            FailoverScheduleStatus::Idle => {
                return next == FailoverScheduleStatus::Ready;
            }
            FailoverScheduleStatus::Warming => {
                match next {
                    FailoverScheduleStatus::Ready | FailoverScheduleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverScheduleStatus::Ready => {
                return next == FailoverScheduleStatus::Draining || next == FailoverScheduleStatus::Stopped;
            }
            FailoverScheduleStatus::Draining => {
                return next == FailoverScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.floor_value * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverSchedule {
        FailoverSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            margin: self.margin,
            source: self.source.clone(),
            floor_value: self.floor_value,
            country: self.country.clone(),
            rate: self.rate,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a failover schedule in status idle.
pub fn new_failover_schedule(code: &str, title: &str) -> FailoverSchedule {
    FailoverSchedule {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        grace_days: 0,
        margin: 0.0,
        source: "".to_string(),
        floor_value: 0.0,
        country: "".to_string(),
        rate: 0.0,
        duration_minutes: 0,
        status: FailoverScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_failover_schedule("code-412", "title-312");
        assert_eq!(item.status, FailoverScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_failover_schedule("code-412", "title-312");
        if item.move_to(FailoverScheduleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverScheduleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_failover_schedule("code-412", "title-312");
        item.status = FailoverScheduleStatus::Stopped;
        assert!(
            item.move_to(FailoverScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
