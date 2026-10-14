use super::*;

use acme_kit::text;

/// Where a federation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FederationMilestoneStatus {
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

impl std::fmt::Display for FederationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationMilestone {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub window_days: i64,
    pub cluster: String,
    pub audience: String,
    pub locale: String,
    pub status: FederationMilestoneStatus,
}

impl FederationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationMilestoneStatus) -> bool {
        match self.status {
            FederationMilestoneStatus::Idle => {
                match next {
                    FederationMilestoneStatus::Warming | FederationMilestoneStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationMilestoneStatus::Warming => {
                return next == FederationMilestoneStatus::Draining || next == FederationMilestoneStatus::Stopped;
            }
            FederationMilestoneStatus::Ready => {
                match next {
                    FederationMilestoneStatus::Draining | FederationMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationMilestoneStatus::Draining => {
                return next == FederationMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "federation milestone", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> FederationMilestone {
        FederationMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            window_days: self.window_days,
            cluster: self.cluster.clone(),
            audience: self.audience.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a federation milestone in status idle.
pub fn new_federation_milestone(code: &str, label: &str) -> FederationMilestone {
    FederationMilestone {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        window_days: 0,
        cluster: "".to_string(),
        audience: "".to_string(),
        locale: "".to_string(),
        status: FederationMilestoneStatus::Idle,
    }
}
