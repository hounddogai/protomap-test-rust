use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a budget burn forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BudgetBurnForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for BudgetBurnForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnForecast {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub budget: f64,
    pub ttl_seconds: i64,
    pub ratio: f64,
    pub rate: f64,
    pub owner_id: String,
    pub attributes: BTreeMap<String, String>,
    pub amount: f64,
    pub name: String,
    pub status: BudgetBurnForecastStatus,
}

impl BudgetBurnForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnForecastStatus) -> bool {
        match self.status {
            BudgetBurnForecastStatus::New => {
                match next {
                    BudgetBurnForecastStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnForecastStatus::Queued => {
                match next {
                    BudgetBurnForecastStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnForecastStatus::Running => {
                return next == BudgetBurnForecastStatus::Failed;
            }
            BudgetBurnForecastStatus::Done => {
                match next {
                    BudgetBurnForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: BudgetBurnForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn forecast", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.amount + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnForecast {
        BudgetBurnForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            budget: self.budget,
            ttl_seconds: self.ttl_seconds,
            ratio: self.ratio,
            rate: self.rate,
            owner_id: self.owner_id.clone(),
            attributes: self.attributes.clone(),
            amount: self.amount,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a budget burn forecast in status new.
pub fn new_budget_burn_forecast(code: &str, title: &str) -> BudgetBurnForecast {
    BudgetBurnForecast {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        budget: 0.0,
        ttl_seconds: 0,
        ratio: 0.0,
        rate: 0.0,
        owner_id: "".to_string(),
        attributes: BTreeMap::new(),
        amount: 0.0,
        name: "".to_string(),
        status: BudgetBurnForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_budget_burn_forecast("code-495", "title-618");
        assert_eq!(item.status, BudgetBurnForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_budget_burn_forecast("code-495", "title-618");
        if item.move_to(BudgetBurnForecastStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnForecastStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_budget_burn_forecast("code-495", "title-618");
        item.status = BudgetBurnForecastStatus::Failed;
        assert!(
            item.move_to(BudgetBurnForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
