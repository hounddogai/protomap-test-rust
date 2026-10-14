use super::*;

use acme_kit::text;

/// Where a action item plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemPlanStatus {
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

impl std::fmt::Display for ActionItemPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item plan as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPlan {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub quota_units: i64,
    pub active: bool,
    pub fingerprint: String,
    pub burst: i64,
    pub status: ActionItemPlanStatus,
}

impl ActionItemPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemPlanStatus) -> bool {
        match self.status {
            ActionItemPlanStatus::Idle => {
                match next {
                    ActionItemPlanStatus::Draining | ActionItemPlanStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemPlanStatus::Warming => {
                match next {
                    ActionItemPlanStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemPlanStatus::Ready => {
                return next == ActionItemPlanStatus::Draining || next == ActionItemPlanStatus::Stopped;
            }
            ActionItemPlanStatus::Draining => {
                return next == ActionItemPlanStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item plan", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.burst as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemPlan {
        ActionItemPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            quota_units: self.quota_units,
            active: self.active,
            fingerprint: self.fingerprint.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item plan in status idle.
pub fn new_action_item_plan(code: &str, name: &str) -> ActionItemPlan {
    ActionItemPlan {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        quota_units: 0,
        active: false,
        fingerprint: "".to_string(),
        burst: 0,
        status: ActionItemPlanStatus::Idle,
    }
}
