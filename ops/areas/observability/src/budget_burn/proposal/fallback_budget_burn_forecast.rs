use super::*;

use acme_kit::text;

/// Where a fallback budget burn forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackBudgetBurnForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackBudgetBurnForecastStatus {
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

impl std::fmt::Display for FallbackBudgetBurnForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback budget burn forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnForecast {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub minimum: f64,
    pub reviewer: String,
    pub title: String,
    pub category: String,
    pub batch_size: i64,
    pub namespace: String,
    pub status: FallbackBudgetBurnForecastStatus,
}

impl FallbackBudgetBurnForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackBudgetBurnForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackBudgetBurnForecastStatus) -> bool {
        match self.status {
            FallbackBudgetBurnForecastStatus::Draft => {
                match next {
                    FallbackBudgetBurnForecastStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBudgetBurnForecastStatus::Pending => {
                return next == FallbackBudgetBurnForecastStatus::Active;
            }
            FallbackBudgetBurnForecastStatus::Active => {
                match next {
                    FallbackBudgetBurnForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackBudgetBurnForecastStatus::Suspended => {
                match next {
                    FallbackBudgetBurnForecastStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackBudgetBurnForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback budget burn forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.batch_size as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackBudgetBurnForecast {
        FallbackBudgetBurnForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            minimum: self.minimum,
            reviewer: self.reviewer.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback budget burn forecast in status draft.
pub fn new_fallback_budget_burn_forecast(code: &str, name: &str) -> FallbackBudgetBurnForecast {
    FallbackBudgetBurnForecast {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        minimum: 0.0,
        reviewer: "".to_string(),
        title: "".to_string(),
        category: "".to_string(),
        batch_size: 0,
        namespace: "".to_string(),
        status: FallbackBudgetBurnForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_budget_burn_forecast("code-134", "name-367");
        assert_eq!(item.status, FallbackBudgetBurnForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_fallback_budget_burn_forecast("code-134", "name-367");
        if item.move_to(FallbackBudgetBurnForecastStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackBudgetBurnForecastStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_budget_burn_forecast("code-134", "name-367");
        item.status = FallbackBudgetBurnForecastStatus::Closed;
        assert!(
            item.move_to(FallbackBudgetBurnForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
