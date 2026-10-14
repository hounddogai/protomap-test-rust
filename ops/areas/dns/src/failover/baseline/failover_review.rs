use super::*;

use acme_kit::text;

/// Where a failover review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FailoverReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FailoverReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverReview {
    pub code: String,
    pub name: String,
    pub label: String,
    pub window_days: i64,
    pub maximum: f64,
    pub audience: String,
    pub priority: i64,
    pub status: FailoverReviewStatus,
}

impl FailoverReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverReviewStatus) -> bool {
        match self.status {
            FailoverReviewStatus::Proposed => {
                return next == FailoverReviewStatus::Approved || next == FailoverReviewStatus::Withdrawn;
            }
            FailoverReviewStatus::Approved => {
                match next {
                    FailoverReviewStatus::Rejected | FailoverReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverReviewStatus::Rejected => {
                return next == FailoverReviewStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.maximum - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverReview {
        FailoverReview {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            maximum: self.maximum,
            audience: self.audience.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a failover review in status proposed.
pub fn new_failover_review(code: &str, name: &str) -> FailoverReview {
    FailoverReview {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        window_days: 0,
        maximum: 0.0,
        audience: "".to_string(),
        priority: 0,
        status: FailoverReviewStatus::Proposed,
    }
}
