use super::*;

use acme_kit::text;

/// Where a retention schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RetentionScheduleStatus {
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

impl std::fmt::Display for RetentionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSchedule {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub approved: bool,
    pub owner_id: String,
    pub amount: f64,
    pub status: RetentionScheduleStatus,
}

impl RetentionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionScheduleStatus) -> bool {
        match self.status {
            RetentionScheduleStatus::Idle => {
                return next == RetentionScheduleStatus::Ready;
            }
            RetentionScheduleStatus::Warming => {
                match next {
                    RetentionScheduleStatus::Draining | RetentionScheduleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionScheduleStatus::Ready => {
                return next == RetentionScheduleStatus::Stopped;
            }
            RetentionScheduleStatus::Draining => {
                match next {
                    RetentionScheduleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RetentionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "retention schedule", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RetentionSchedule {
        RetentionSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            approved: self.approved,
            owner_id: self.owner_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a retention schedule in status idle.
pub fn new_retention_schedule(code: &str, title: &str) -> RetentionSchedule {
    RetentionSchedule {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        approved: false,
        owner_id: "".to_string(),
        amount: 0.0,
        status: RetentionScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_retention_schedule("code-44", "title-334");
        assert_eq!(item.status, RetentionScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_retention_schedule("code-44", "title-334");
        if item.move_to(RetentionScheduleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionScheduleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_retention_schedule("code-44", "title-334");
        item.status = RetentionScheduleStatus::Stopped;
        assert!(
            item.move_to(RetentionScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
