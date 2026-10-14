use super::*;

use acme_kit::text;

/// Where a budget burn estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BudgetBurnEstimateStatus {
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

impl std::fmt::Display for BudgetBurnEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnEstimate {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub rate: f64,
    pub name: String,
    pub retries: i64,
    pub tenant_id: String,
    pub port: i64,
    pub attempts: i64,
    pub cost: f64,
    pub limit: i64,
    pub status: BudgetBurnEstimateStatus,
}

impl BudgetBurnEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnEstimateStatus) -> bool {
        match self.status {
            BudgetBurnEstimateStatus::Open => {
                return next == BudgetBurnEstimateStatus::Archived;
            }
            BudgetBurnEstimateStatus::InReview => {
                return next == BudgetBurnEstimateStatus::Resolved;
            }
            BudgetBurnEstimateStatus::Resolved => {
                match next {
                    BudgetBurnEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnEstimateStatus::Reopened => {
                match next {
                    BudgetBurnEstimateStatus::Archived => {
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

    pub fn move_to(&mut self, next: BudgetBurnEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.attempts as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnEstimate {
        BudgetBurnEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            rate: self.rate,
            name: self.name.clone(),
            retries: self.retries,
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            attempts: self.attempts,
            cost: self.cost,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a budget burn estimate in status open.
pub fn new_budget_burn_estimate(code: &str, title: &str) -> BudgetBurnEstimate {
    BudgetBurnEstimate {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        rate: 0.0,
        name: "".to_string(),
        retries: 0,
        tenant_id: "".to_string(),
        port: 0,
        attempts: 0,
        cost: 0.0,
        limit: 0,
        status: BudgetBurnEstimateStatus::Open,
    }
}
