use super::*;

use acme_kit::text;

/// Where a manual limit budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualLimitBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualLimitBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualLimitBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual limit budget as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitBudget {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub approved: bool,
    pub country: String,
    pub discount: f64,
    pub status: ManualLimitBudgetStatus,
}

impl ManualLimitBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualLimitBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualLimitBudgetStatus) -> bool {
        match self.status {
            ManualLimitBudgetStatus::Proposed => {
                match next {
                    ManualLimitBudgetStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualLimitBudgetStatus::Approved => {
                match next {
                    ManualLimitBudgetStatus::Rejected | ManualLimitBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualLimitBudgetStatus::Rejected => {
                match next {
                    ManualLimitBudgetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualLimitBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual limit budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> ManualLimitBudget {
        ManualLimitBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            approved: self.approved,
            country: self.country.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual limit budget in status proposed.
pub fn new_manual_limit_budget(code: &str, name: &str) -> ManualLimitBudget {
    ManualLimitBudget {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        approved: false,
        country: "".to_string(),
        discount: 0.0,
        status: ManualLimitBudgetStatus::Proposed,
    }
}
