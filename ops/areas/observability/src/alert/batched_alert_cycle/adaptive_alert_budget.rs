use super::*;

use acme_kit::text;

/// Where a adaptive alert budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAlertBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveAlertBudgetStatus {
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

impl std::fmt::Display for AdaptiveAlertBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive alert budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertBudget {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub unit: String,
    pub quota_units: i64,
    pub summary: String,
    pub currency: String,
    pub weight: f64,
    pub phone: String,
    pub status: AdaptiveAlertBudgetStatus,
}

impl AdaptiveAlertBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAlertBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAlertBudgetStatus) -> bool {
        match self.status {
            AdaptiveAlertBudgetStatus::Open => {
                match next {
                    AdaptiveAlertBudgetStatus::Resolved | AdaptiveAlertBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAlertBudgetStatus::InReview => {
                match next {
                    AdaptiveAlertBudgetStatus::Resolved | AdaptiveAlertBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAlertBudgetStatus::Resolved => {
                match next {
                    AdaptiveAlertBudgetStatus::Reopened | AdaptiveAlertBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAlertBudgetStatus::Reopened => {
                match next {
                    AdaptiveAlertBudgetStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveAlertBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive alert budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive alert budget", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.weight - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAlertBudget {
        AdaptiveAlertBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            unit: self.unit.clone(),
            quota_units: self.quota_units,
            summary: self.summary.clone(),
            currency: self.currency.clone(),
            weight: self.weight,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a adaptive alert budget in status open.
pub fn new_adaptive_alert_budget(code: &str, name: &str) -> AdaptiveAlertBudget {
    AdaptiveAlertBudget {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        unit: "".to_string(),
        quota_units: 0,
        summary: "".to_string(),
        currency: "".to_string(),
        weight: 0.0,
        phone: "".to_string(),
        status: AdaptiveAlertBudgetStatus::Open,
    }
}
