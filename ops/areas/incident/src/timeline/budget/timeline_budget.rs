use super::*;

use acme_kit::text;

/// Where a timeline budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TimelineBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TimelineBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline budget as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudget {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub margin: f64,
    pub window_days: i64,
    pub description: String,
    pub grace_days: i64,
    pub key_id: String,
    pub audience: String,
    pub status: TimelineBudgetStatus,
}

impl TimelineBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineBudgetStatus) -> bool {
        match self.status {
            TimelineBudgetStatus::Proposed => {
                return next == TimelineBudgetStatus::Approved;
            }
            TimelineBudgetStatus::Approved => {
                match next {
                    TimelineBudgetStatus::Rejected | TimelineBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineBudgetStatus::Rejected => {
                return next == TimelineBudgetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.grace_days as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineBudget {
        TimelineBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            margin: self.margin,
            window_days: self.window_days,
            description: self.description.clone(),
            grace_days: self.grace_days,
            key_id: self.key_id.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a timeline budget in status proposed.
pub fn new_timeline_budget(code: &str, title: &str) -> TimelineBudget {
    TimelineBudget {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        margin: 0.0,
        window_days: 0,
        description: "".to_string(),
        grace_days: 0,
        key_id: "".to_string(),
        audience: "".to_string(),
        status: TimelineBudgetStatus::Proposed,
    }
}
