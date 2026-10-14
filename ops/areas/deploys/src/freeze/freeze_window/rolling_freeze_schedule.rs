use super::*;

use acme_kit::text;

/// Where a rolling freeze schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFreezeScheduleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingFreezeScheduleStatus {
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

impl std::fmt::Display for RollingFreezeScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling freeze schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFreezeSchedule {
    pub code: String,
    pub title: String,
    pub country: String,
    pub version: i64,
    pub pinned: bool,
    pub locale: String,
    pub created_at: i64,
    pub amount: f64,
    pub status: RollingFreezeScheduleStatus,
}

impl RollingFreezeSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFreezeScheduleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFreezeScheduleStatus) -> bool {
        match self.status {
            RollingFreezeScheduleStatus::Planned => {
                return next == RollingFreezeScheduleStatus::InProgress
                    || next == RollingFreezeScheduleStatus::Cancelled;
            }
            RollingFreezeScheduleStatus::Scheduled => {
                return next == RollingFreezeScheduleStatus::Completed
                    || next == RollingFreezeScheduleStatus::Cancelled;
            }
            RollingFreezeScheduleStatus::InProgress => {
                return next == RollingFreezeScheduleStatus::Completed
                    || next == RollingFreezeScheduleStatus::Cancelled;
            }
            RollingFreezeScheduleStatus::Completed => {
                match next {
                    RollingFreezeScheduleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingFreezeScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling freeze schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.created_at as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingFreezeSchedule {
        RollingFreezeSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            version: self.version,
            pinned: self.pinned,
            locale: self.locale.clone(),
            created_at: self.created_at,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling freeze schedule in status planned.
pub fn new_rolling_freeze_schedule(code: &str, title: &str) -> RollingFreezeSchedule {
    RollingFreezeSchedule {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        version: 0,
        pinned: false,
        locale: "".to_string(),
        created_at: 0,
        amount: 0.0,
        status: RollingFreezeScheduleStatus::Planned,
    }
}
