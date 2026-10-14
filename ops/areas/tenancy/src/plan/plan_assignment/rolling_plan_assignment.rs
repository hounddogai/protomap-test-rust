use super::*;

use acme_kit::text;

/// Where a rolling plan assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPlanAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingPlanAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingPlanAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling plan assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanAssignment {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub weight: f64,
    pub principal: String,
    pub host: String,
    pub channel: String,
    pub status: RollingPlanAssignmentStatus,
}

impl RollingPlanAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPlanAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPlanAssignmentStatus) -> bool {
        match self.status {
            RollingPlanAssignmentStatus::Created => {
                return next == RollingPlanAssignmentStatus::Validated;
            }
            RollingPlanAssignmentStatus::Validated => {
                return next == RollingPlanAssignmentStatus::Retired;
            }
            RollingPlanAssignmentStatus::Published => {
                return next == RollingPlanAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPlanAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling plan assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling plan assignment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> RollingPlanAssignment {
        RollingPlanAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            weight: self.weight,
            principal: self.principal.clone(),
            host: self.host.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rolling plan assignment in status created.
pub fn new_rolling_plan_assignment(code: &str, title: &str) -> RollingPlanAssignment {
    RollingPlanAssignment {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        weight: 0.0,
        principal: "".to_string(),
        host: "".to_string(),
        channel: "".to_string(),
        status: RollingPlanAssignmentStatus::Created,
    }
}
