use super::*;

use acme_kit::text;

/// Where a rollout milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RolloutMilestoneStatus {
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

impl std::fmt::Display for RolloutMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMilestone {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub external_ref: String,
    pub rotation_days: i64,
    pub quota_units: i64,
    pub floor_value: f64,
    pub principal: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub status: RolloutMilestoneStatus,
}

impl RolloutMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutMilestoneStatus) -> bool {
        match self.status {
            RolloutMilestoneStatus::Idle => {
                return next == RolloutMilestoneStatus::Ready || next == RolloutMilestoneStatus::Stopped;
            }
            RolloutMilestoneStatus::Warming => {
                return next == RolloutMilestoneStatus::Ready || next == RolloutMilestoneStatus::Draining;
            }
            RolloutMilestoneStatus::Ready => {
                match next {
                    RolloutMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutMilestoneStatus::Draining => {
                return next == RolloutMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutMilestone {
        RolloutMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            quota_units: self.quota_units,
            floor_value: self.floor_value,
            principal: self.principal.clone(),
            duration_minutes: self.duration_minutes,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rollout milestone in status idle.
pub fn new_rollout_milestone(code: &str, title: &str) -> RolloutMilestone {
    RolloutMilestone {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        external_ref: "".to_string(),
        rotation_days: 0,
        quota_units: 0,
        floor_value: 0.0,
        principal: "".to_string(),
        duration_minutes: 0,
        capacity: 0,
        status: RolloutMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rollout_milestone("code-829", "title-824");
        assert_eq!(item.status, RolloutMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_rollout_milestone("code-829", "title-824");
        if item.move_to(RolloutMilestoneStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutMilestoneStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rollout_milestone("code-829", "title-824");
        item.status = RolloutMilestoneStatus::Stopped;
        assert!(
            item.move_to(RolloutMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
