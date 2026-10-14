use super::*;

use acme_kit::text;

/// Where a binding milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingMilestoneStatus {
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

impl std::fmt::Display for BindingMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingMilestone {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub rotation_days: i64,
    pub quota_units: i64,
    pub grace_days: i64,
    pub ratio: f64,
    pub attempts: i64,
    pub region: String,
    pub namespace: String,
    pub status: BindingMilestoneStatus,
}

impl BindingMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingMilestoneStatus) -> bool {
        match self.status {
            BindingMilestoneStatus::Idle => {
                return next == BindingMilestoneStatus::Ready;
            }
            BindingMilestoneStatus::Warming => {
                return next == BindingMilestoneStatus::Ready || next == BindingMilestoneStatus::Draining;
            }
            BindingMilestoneStatus::Ready => {
                return next == BindingMilestoneStatus::Stopped;
            }
            BindingMilestoneStatus::Draining => {
                return next == BindingMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "binding milestone", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.attempts as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingMilestone {
        BindingMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            rotation_days: self.rotation_days,
            quota_units: self.quota_units,
            grace_days: self.grace_days,
            ratio: self.ratio,
            attempts: self.attempts,
            region: self.region.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a binding milestone in status idle.
pub fn new_binding_milestone(code: &str, title: &str) -> BindingMilestone {
    BindingMilestone {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        rotation_days: 0,
        quota_units: 0,
        grace_days: 0,
        ratio: 0.0,
        attempts: 0,
        region: "".to_string(),
        namespace: "".to_string(),
        status: BindingMilestoneStatus::Idle,
    }
}
