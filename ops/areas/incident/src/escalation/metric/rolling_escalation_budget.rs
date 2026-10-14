use super::*;

use acme_kit::text;

/// Where a rolling escalation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEscalationBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingEscalationBudgetStatus {
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

impl std::fmt::Display for RollingEscalationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling escalation budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationBudget {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub issuer: String,
    pub capacity: i64,
    pub cost: f64,
    pub window_days: i64,
    pub rotation_days: i64,
    pub margin: f64,
    pub status: RollingEscalationBudgetStatus,
}

impl RollingEscalationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEscalationBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEscalationBudgetStatus) -> bool {
        match self.status {
            RollingEscalationBudgetStatus::Open => {
                return next == RollingEscalationBudgetStatus::InReview
                    || next == RollingEscalationBudgetStatus::Resolved;
            }
            RollingEscalationBudgetStatus::InReview => {
                match next {
                    RollingEscalationBudgetStatus::Resolved | RollingEscalationBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEscalationBudgetStatus::Resolved => {
                match next {
                    RollingEscalationBudgetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEscalationBudgetStatus::Reopened => {
                return next == RollingEscalationBudgetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingEscalationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling escalation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.window_days as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingEscalationBudget {
        RollingEscalationBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            issuer: self.issuer.clone(),
            capacity: self.capacity,
            cost: self.cost,
            window_days: self.window_days,
            rotation_days: self.rotation_days,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling escalation budget in status open.
pub fn new_rolling_escalation_budget(code: &str, label: &str) -> RollingEscalationBudget {
    RollingEscalationBudget {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        issuer: "".to_string(),
        capacity: 0,
        cost: 0.0,
        window_days: 0,
        rotation_days: 0,
        margin: 0.0,
        status: RollingEscalationBudgetStatus::Open,
    }
}
