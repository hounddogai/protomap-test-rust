use super::*;

use acme_kit::text;

/// Where a plan rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PlanRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PlanRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRollup {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub category: String,
    pub locale: String,
    pub grace_days: i64,
    pub status: PlanRollupStatus,
}

impl PlanRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanRollupStatus) -> bool {
        match self.status {
            PlanRollupStatus::Open => {
                match next {
                    PlanRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanRollupStatus::InReview => {
                return next == PlanRollupStatus::Resolved || next == PlanRollupStatus::Archived;
            }
            PlanRollupStatus::Resolved => {
                match next {
                    PlanRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanRollupStatus::Reopened => {
                match next {
                    PlanRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: PlanRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "plan rollup", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanRollup {
        PlanRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            category: self.category.clone(),
            locale: self.locale.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a plan rollup in status open.
pub fn new_plan_rollup(code: &str, label: &str) -> PlanRollup {
    PlanRollup {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        category: "".to_string(),
        locale: "".to_string(),
        grace_days: 0,
        status: PlanRollupStatus::Open,
    }
}
