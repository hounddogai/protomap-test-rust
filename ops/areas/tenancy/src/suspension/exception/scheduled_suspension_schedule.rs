use super::*;

use acme_kit::text;

/// Where a scheduled suspension schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSuspensionScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledSuspensionScheduleStatus {
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

impl std::fmt::Display for ScheduledSuspensionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled suspension schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionSchedule {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub reviewer: String,
    pub segment: String,
    pub limit: i64,
    pub status: ScheduledSuspensionScheduleStatus,
}

impl ScheduledSuspensionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSuspensionScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSuspensionScheduleStatus) -> bool {
        match self.status {
            ScheduledSuspensionScheduleStatus::Idle => {
                match next {
                    ScheduledSuspensionScheduleStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSuspensionScheduleStatus::Warming => {
                return next == ScheduledSuspensionScheduleStatus::Ready;
            }
            ScheduledSuspensionScheduleStatus::Ready => {
                return next == ScheduledSuspensionScheduleStatus::Draining;
            }
            ScheduledSuspensionScheduleStatus::Draining => {
                return next == ScheduledSuspensionScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSuspensionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled suspension schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled suspension schedule", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.limit as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSuspensionSchedule {
        ScheduledSuspensionSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            reviewer: self.reviewer.clone(),
            segment: self.segment.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled suspension schedule in status idle.
pub fn new_scheduled_suspension_schedule(code: &str, label: &str) -> ScheduledSuspensionSchedule {
    ScheduledSuspensionSchedule {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        reviewer: "".to_string(),
        segment: "".to_string(),
        limit: 0,
        status: ScheduledSuspensionScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_suspension_schedule("code-838", "label-59");
        assert_eq!(item.status, ScheduledSuspensionScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_suspension_schedule("code-838", "label-59");
        if item.move_to(ScheduledSuspensionScheduleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSuspensionScheduleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_suspension_schedule("code-838", "label-59");
        item.status = ScheduledSuspensionScheduleStatus::Stopped;
        assert!(
            item.move_to(ScheduledSuspensionScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
