use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled sink schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSinkScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledSinkScheduleStatus {
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

impl std::fmt::Display for ScheduledSinkScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled sink schedule as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkSchedule {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub region: String,
    pub cost: f64,
    pub score: f64,
    pub status: ScheduledSinkScheduleStatus,
}

impl ScheduledSinkSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSinkScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSinkScheduleStatus) -> bool {
        match self.status {
            ScheduledSinkScheduleStatus::New => {
                return next == ScheduledSinkScheduleStatus::Queued || next == ScheduledSinkScheduleStatus::Running;
            }
            ScheduledSinkScheduleStatus::Queued => {
                match next {
                    ScheduledSinkScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSinkScheduleStatus::Running => {
                match next {
                    ScheduledSinkScheduleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSinkScheduleStatus::Done => {
                return next == ScheduledSinkScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSinkScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled sink schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled sink schedule", self.code)
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

    pub fn weighted_total(&self) -> f64 {
        (self.cost * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSinkSchedule {
        ScheduledSinkSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            region: self.region.clone(),
            cost: self.cost,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a scheduled sink schedule in status new.
pub fn new_scheduled_sink_schedule(code: &str, name: &str) -> ScheduledSinkSchedule {
    ScheduledSinkSchedule {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        region: "".to_string(),
        cost: 0.0,
        score: 0.0,
        status: ScheduledSinkScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_sink_schedule("code-128", "name-820");
        assert_eq!(item.status, ScheduledSinkScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_scheduled_sink_schedule("code-128", "name-820");
        if item.move_to(ScheduledSinkScheduleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSinkScheduleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_sink_schedule("code-128", "name-820");
        item.status = ScheduledSinkScheduleStatus::Failed;
        assert!(
            item.move_to(ScheduledSinkScheduleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
