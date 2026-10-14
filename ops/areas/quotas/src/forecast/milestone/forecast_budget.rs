use super::*;

use acme_kit::text;

/// Where a forecast budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ForecastBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBudget {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub cost: f64,
    pub scope: String,
    pub status: ForecastBudgetStatus,
}

impl ForecastBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastBudgetStatus) -> bool {
        match self.status {
            ForecastBudgetStatus::Draft => {
                match next {
                    ForecastBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastBudgetStatus::Pending => {
                match next {
                    ForecastBudgetStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastBudgetStatus::Active => {
                return next == ForecastBudgetStatus::Suspended || next == ForecastBudgetStatus::Closed;
            }
            ForecastBudgetStatus::Suspended => {
                return next == ForecastBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.cost + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastBudget {
        ForecastBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            cost: self.cost,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a forecast budget in status draft.
pub fn new_forecast_budget(code: &str, label: &str) -> ForecastBudget {
    ForecastBudget {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        cost: 0.0,
        scope: "".to_string(),
        status: ForecastBudgetStatus::Draft,
    }
}
