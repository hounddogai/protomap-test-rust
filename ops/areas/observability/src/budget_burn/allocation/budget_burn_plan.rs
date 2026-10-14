use super::*;

use acme_kit::text;

/// Where a budget burn plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BudgetBurnPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BudgetBurnPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn plan as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlan {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub pinned: bool,
    pub phone: String,
    pub threshold: f64,
    pub cluster: String,
    pub tenant_id: String,
    pub status: BudgetBurnPlanStatus,
}

impl BudgetBurnPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnPlanStatus) -> bool {
        match self.status {
            BudgetBurnPlanStatus::Created => {
                return next == BudgetBurnPlanStatus::Published || next == BudgetBurnPlanStatus::Retired;
            }
            BudgetBurnPlanStatus::Validated => {
                match next {
                    BudgetBurnPlanStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnPlanStatus::Published => {
                match next {
                    BudgetBurnPlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: BudgetBurnPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> BudgetBurnPlan {
        BudgetBurnPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            pinned: self.pinned,
            phone: self.phone.clone(),
            threshold: self.threshold,
            cluster: self.cluster.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a budget burn plan in status created.
pub fn new_budget_burn_plan(code: &str, label: &str) -> BudgetBurnPlan {
    BudgetBurnPlan {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        pinned: false,
        phone: "".to_string(),
        threshold: 0.0,
        cluster: "".to_string(),
        tenant_id: "".to_string(),
        status: BudgetBurnPlanStatus::Created,
    }
}
