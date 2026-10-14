use super::*;

use acme_kit::text;

/// Where a weighted timeline plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTimelinePlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedTimelinePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for WeightedTimelinePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted timeline plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlan {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub version: i64,
    pub timezone: String,
    pub budget: f64,
    pub archived: bool,
    pub attempts: i64,
    pub status: WeightedTimelinePlanStatus,
}

impl WeightedTimelinePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTimelinePlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTimelinePlanStatus) -> bool {
        match self.status {
            WeightedTimelinePlanStatus::Planned => {
                match next {
                    WeightedTimelinePlanStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTimelinePlanStatus::Scheduled => {
                match next {
                    WeightedTimelinePlanStatus::InProgress | WeightedTimelinePlanStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTimelinePlanStatus::InProgress => {
                match next {
                    WeightedTimelinePlanStatus::Completed | WeightedTimelinePlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTimelinePlanStatus::Completed => {
                return next == WeightedTimelinePlanStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTimelinePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted timeline plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.attempts as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTimelinePlan {
        WeightedTimelinePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            version: self.version,
            timezone: self.timezone.clone(),
            budget: self.budget,
            archived: self.archived,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted timeline plan in status planned.
pub fn new_weighted_timeline_plan(code: &str, label: &str) -> WeightedTimelinePlan {
    WeightedTimelinePlan {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        version: 0,
        timezone: "".to_string(),
        budget: 0.0,
        archived: false,
        attempts: 0,
        status: WeightedTimelinePlanStatus::Planned,
    }
}
