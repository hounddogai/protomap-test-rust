use super::*;

use acme_kit::text;

/// Where a rolling token schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTokenScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingTokenScheduleStatus {
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

impl std::fmt::Display for RollingTokenScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling token schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenSchedule {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub retries: i64,
    pub host: String,
    pub burst: i64,
    pub unit: String,
    pub status: RollingTokenScheduleStatus,
}

impl RollingTokenSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTokenScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTokenScheduleStatus) -> bool {
        match self.status {
            RollingTokenScheduleStatus::Idle => {
                match next {
                    RollingTokenScheduleStatus::Warming | RollingTokenScheduleStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTokenScheduleStatus::Warming => {
                match next {
                    RollingTokenScheduleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTokenScheduleStatus::Ready => {
                match next {
                    RollingTokenScheduleStatus::Draining | RollingTokenScheduleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTokenScheduleStatus::Draining => {
                return next == RollingTokenScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingTokenScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling token schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling token schedule", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTokenSchedule {
        RollingTokenSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            retries: self.retries,
            host: self.host.clone(),
            burst: self.burst,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling token schedule in status idle.
pub fn new_rolling_token_schedule(code: &str, label: &str) -> RollingTokenSchedule {
    RollingTokenSchedule {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        retries: 0,
        host: "".to_string(),
        burst: 0,
        unit: "".to_string(),
        status: RollingTokenScheduleStatus::Idle,
    }
}
