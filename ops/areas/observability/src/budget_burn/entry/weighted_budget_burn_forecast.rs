use super::*;

use acme_kit::text;

/// Where a weighted budget burn forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedBudgetBurnForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedBudgetBurnForecastStatus {
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

impl std::fmt::Display for WeightedBudgetBurnForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted budget burn forecast as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnForecast {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub ttl_seconds: i64,
    pub namespace: String,
    pub port: i64,
    pub status: WeightedBudgetBurnForecastStatus,
}

impl WeightedBudgetBurnForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedBudgetBurnForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedBudgetBurnForecastStatus) -> bool {
        match self.status {
            WeightedBudgetBurnForecastStatus::Planned => {
                return next == WeightedBudgetBurnForecastStatus::Completed
                    || next == WeightedBudgetBurnForecastStatus::Cancelled;
            }
            WeightedBudgetBurnForecastStatus::Scheduled => {
                match next {
                    WeightedBudgetBurnForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnForecastStatus::InProgress => {
                match next {
                    WeightedBudgetBurnForecastStatus::Completed | WeightedBudgetBurnForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedBudgetBurnForecastStatus::Completed => {
                return next == WeightedBudgetBurnForecastStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedBudgetBurnForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted budget burn forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedBudgetBurnForecast {
        WeightedBudgetBurnForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            ttl_seconds: self.ttl_seconds,
            namespace: self.namespace.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a weighted budget burn forecast in status planned.
pub fn new_weighted_budget_burn_forecast(code: &str, name: &str) -> WeightedBudgetBurnForecast {
    WeightedBudgetBurnForecast {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        ttl_seconds: 0,
        namespace: "".to_string(),
        port: 0,
        status: WeightedBudgetBurnForecastStatus::Planned,
    }
}
