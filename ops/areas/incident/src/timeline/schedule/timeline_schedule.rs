use super::*;

use acme_kit::text;

/// Where a timeline schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TimelineScheduleStatus {
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

impl std::fmt::Display for TimelineScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline schedule as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSchedule {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub grace_days: i64,
    pub amount: f64,
    pub channel: String,
    pub unit: String,
    pub cost: f64,
    pub expires_at: i64,
    pub quota_units: i64,
    pub quantity: i64,
    pub status: TimelineScheduleStatus,
}

impl TimelineSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineScheduleStatus) -> bool {
        match self.status {
            TimelineScheduleStatus::Idle => {
                match next {
                    TimelineScheduleStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineScheduleStatus::Warming => {
                match next {
                    TimelineScheduleStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineScheduleStatus::Ready => {
                return next == TimelineScheduleStatus::Draining || next == TimelineScheduleStatus::Stopped;
            }
            TimelineScheduleStatus::Draining => {
                match next {
                    TimelineScheduleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TimelineScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        (self.cost * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineSchedule {
        TimelineSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            grace_days: self.grace_days,
            amount: self.amount,
            channel: self.channel.clone(),
            unit: self.unit.clone(),
            cost: self.cost,
            expires_at: self.expires_at,
            quota_units: self.quota_units,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a timeline schedule in status idle.
pub fn new_timeline_schedule(code: &str, name: &str) -> TimelineSchedule {
    TimelineSchedule {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        grace_days: 0,
        amount: 0.0,
        channel: "".to_string(),
        unit: "".to_string(),
        cost: 0.0,
        expires_at: 0,
        quota_units: 0,
        quantity: 0,
        status: TimelineScheduleStatus::Idle,
    }
}
