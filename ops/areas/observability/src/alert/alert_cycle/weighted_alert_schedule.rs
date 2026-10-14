use super::*;

use acme_kit::text;

/// Where a weighted alert schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedAlertScheduleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedAlertScheduleStatus {
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

impl std::fmt::Display for WeightedAlertScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted alert schedule as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertSchedule {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub status: WeightedAlertScheduleStatus,
}

impl WeightedAlertSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedAlertScheduleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedAlertScheduleStatus) -> bool {
        match self.status {
            WeightedAlertScheduleStatus::Planned => {
                match next {
                    WeightedAlertScheduleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedAlertScheduleStatus::Scheduled => {
                match next {
                    WeightedAlertScheduleStatus::Completed | WeightedAlertScheduleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedAlertScheduleStatus::InProgress => {
                match next {
                    WeightedAlertScheduleStatus::Completed | WeightedAlertScheduleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedAlertScheduleStatus::Completed => {
                return next == WeightedAlertScheduleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedAlertScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted alert schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted alert schedule", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> WeightedAlertSchedule {
        WeightedAlertSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            duration_minutes: self.duration_minutes,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted alert schedule in status planned.
pub fn new_weighted_alert_schedule(code: &str, name: &str) -> WeightedAlertSchedule {
    WeightedAlertSchedule {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        duration_minutes: 0,
        tags: Vec::new(),
        status: WeightedAlertScheduleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_alert_schedule("code-579", "name-92");
        assert_eq!(item.status, WeightedAlertScheduleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_weighted_alert_schedule("code-579", "name-92");
        if item.move_to(WeightedAlertScheduleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedAlertScheduleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_alert_schedule("code-579", "name-92");
        item.status = WeightedAlertScheduleStatus::Cancelled;
        assert!(
            item.move_to(WeightedAlertScheduleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
