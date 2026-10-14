use super::*;

use acme_kit::text;

/// Where a scheduled binding schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledBindingScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledBindingScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledBindingScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled binding schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingSchedule {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub locale: String,
    pub budget: f64,
    pub threshold: f64,
    pub cluster: String,
    pub score: f64,
    pub status: ScheduledBindingScheduleStatus,
}

impl ScheduledBindingSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledBindingScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledBindingScheduleStatus) -> bool {
        match self.status {
            ScheduledBindingScheduleStatus::Created => {
                return next == ScheduledBindingScheduleStatus::Retired;
            }
            ScheduledBindingScheduleStatus::Validated => {
                return next == ScheduledBindingScheduleStatus::Published
                    || next == ScheduledBindingScheduleStatus::Retired;
            }
            ScheduledBindingScheduleStatus::Published => {
                match next {
                    ScheduledBindingScheduleStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledBindingScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled binding schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        (self.score * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledBindingSchedule {
        ScheduledBindingSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            locale: self.locale.clone(),
            budget: self.budget,
            threshold: self.threshold,
            cluster: self.cluster.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled binding schedule in status created.
pub fn new_scheduled_binding_schedule(code: &str, title: &str) -> ScheduledBindingSchedule {
    ScheduledBindingSchedule {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        locale: "".to_string(),
        budget: 0.0,
        threshold: 0.0,
        cluster: "".to_string(),
        score: 0.0,
        status: ScheduledBindingScheduleStatus::Created,
    }
}
