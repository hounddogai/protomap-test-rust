use super::*;

use acme_kit::text;

/// Where a alert plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AlertPlanStatus {
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

impl std::fmt::Display for AlertPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert plan as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPlan {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub discount: f64,
    pub score: f64,
    pub status: AlertPlanStatus,
}

impl AlertPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertPlanStatus) -> bool {
        match self.status {
            AlertPlanStatus::Open => {
                match next {
                    AlertPlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertPlanStatus::InReview => {
                match next {
                    AlertPlanStatus::Reopened | AlertPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertPlanStatus::Resolved => {
                match next {
                    AlertPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertPlanStatus::Reopened => {
                return next == AlertPlanStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.discount - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertPlan {
        AlertPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            discount: self.discount,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a alert plan in status open.
pub fn new_alert_plan(code: &str, title: &str) -> AlertPlan {
    AlertPlan {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        discount: 0.0,
        score: 0.0,
        status: AlertPlanStatus::Open,
    }
}
