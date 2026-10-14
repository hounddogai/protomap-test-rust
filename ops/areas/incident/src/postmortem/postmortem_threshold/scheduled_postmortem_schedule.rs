use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled postmortem schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPostmortemScheduleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledPostmortemScheduleStatus {
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

impl std::fmt::Display for ScheduledPostmortemScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled postmortem schedule as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemSchedule {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub updated_at: i64,
    pub channel: String,
    pub threshold: f64,
    pub attributes: BTreeMap<String, String>,
    pub timezone: String,
    pub port: i64,
    pub status: ScheduledPostmortemScheduleStatus,
}

impl ScheduledPostmortemSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPostmortemScheduleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPostmortemScheduleStatus) -> bool {
        match self.status {
            ScheduledPostmortemScheduleStatus::Planned => {
                return next == ScheduledPostmortemScheduleStatus::InProgress
                    || next == ScheduledPostmortemScheduleStatus::Cancelled;
            }
            ScheduledPostmortemScheduleStatus::Scheduled => {
                match next {
                    ScheduledPostmortemScheduleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPostmortemScheduleStatus::InProgress => {
                return next == ScheduledPostmortemScheduleStatus::Completed
                    || next == ScheduledPostmortemScheduleStatus::Cancelled;
            }
            ScheduledPostmortemScheduleStatus::Completed => {
                return next == ScheduledPostmortemScheduleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledPostmortemScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled postmortem schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled postmortem schedule", self.code)
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
        ((self.port as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledPostmortemSchedule {
        ScheduledPostmortemSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            updated_at: self.updated_at,
            channel: self.channel.clone(),
            threshold: self.threshold,
            attributes: self.attributes.clone(),
            timezone: self.timezone.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled postmortem schedule in status planned.
pub fn new_scheduled_postmortem_schedule(code: &str, title: &str) -> ScheduledPostmortemSchedule {
    ScheduledPostmortemSchedule {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        updated_at: 0,
        channel: "".to_string(),
        threshold: 0.0,
        attributes: BTreeMap::new(),
        timezone: "".to_string(),
        port: 0,
        status: ScheduledPostmortemScheduleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_postmortem_schedule("code-396", "title-170");
        assert_eq!(item.status, ScheduledPostmortemScheduleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_scheduled_postmortem_schedule("code-396", "title-170");
        if item.move_to(ScheduledPostmortemScheduleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledPostmortemScheduleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_postmortem_schedule("code-396", "title-170");
        item.status = ScheduledPostmortemScheduleStatus::Cancelled;
        assert!(
            item.move_to(ScheduledPostmortemScheduleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
