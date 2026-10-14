use super::*;

use acme_kit::text;

/// Where a seasonal budget burn plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalBudgetBurnPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalBudgetBurnPlanStatus {
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

impl std::fmt::Display for SeasonalBudgetBurnPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal budget burn plan as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnPlan {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub retries: i64,
    pub version: i64,
    pub archived: bool,
    pub currency: String,
    pub fingerprint: String,
    pub phone: String,
    pub notes: String,
    pub segment: String,
    pub status: SeasonalBudgetBurnPlanStatus,
}

impl SeasonalBudgetBurnPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalBudgetBurnPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalBudgetBurnPlanStatus) -> bool {
        match self.status {
            SeasonalBudgetBurnPlanStatus::Open => {
                return next == SeasonalBudgetBurnPlanStatus::Reopened
                    || next == SeasonalBudgetBurnPlanStatus::Archived;
            }
            SeasonalBudgetBurnPlanStatus::InReview => {
                match next {
                    SeasonalBudgetBurnPlanStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalBudgetBurnPlanStatus::Resolved => {
                match next {
                    SeasonalBudgetBurnPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalBudgetBurnPlanStatus::Reopened => {
                match next {
                    SeasonalBudgetBurnPlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalBudgetBurnPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal budget burn plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalBudgetBurnPlan {
        SeasonalBudgetBurnPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            retries: self.retries,
            version: self.version,
            archived: self.archived,
            currency: self.currency.clone(),
            fingerprint: self.fingerprint.clone(),
            phone: self.phone.clone(),
            notes: self.notes.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a seasonal budget burn plan in status open.
pub fn new_seasonal_budget_burn_plan(code: &str, label: &str) -> SeasonalBudgetBurnPlan {
    SeasonalBudgetBurnPlan {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        retries: 0,
        version: 0,
        archived: false,
        currency: "".to_string(),
        fingerprint: "".to_string(),
        phone: "".to_string(),
        notes: "".to_string(),
        segment: "".to_string(),
        status: SeasonalBudgetBurnPlanStatus::Open,
    }
}
