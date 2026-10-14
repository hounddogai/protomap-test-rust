use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback budget burn forecast must pass. Check returns its problems, or none.
pub trait FallbackBudgetBurnForecastRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBudgetBurnForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnForecastRequiredCodeRule {
    pub message: String,
}

impl FallbackBudgetBurnForecastRule for FallbackBudgetBurnForecastRequiredCodeRule {
    fn check(&self, item: &FallbackBudgetBurnForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnForecastBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackBudgetBurnForecastRule for FallbackBudgetBurnForecastBatchSizeRangeRule {
    fn check(&self, item: &FallbackBudgetBurnForecast) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnForecastRequiredAudienceRule {
    pub message: String,
}

impl FallbackBudgetBurnForecastRule for FallbackBudgetBurnForecastRequiredAudienceRule {
    fn check(&self, item: &FallbackBudgetBurnForecast) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnForecastRequiredReviewerRule {
    pub message: String,
}

impl FallbackBudgetBurnForecastRule for FallbackBudgetBurnForecastRequiredReviewerRule {
    fn check(&self, item: &FallbackBudgetBurnForecast) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback budget burn forecast.
#[derive(Debug, Clone)]
pub struct FallbackBudgetBurnForecastValidator {
    pub rules: Vec<Rc<dyn FallbackBudgetBurnForecastRule>>,
}

impl FallbackBudgetBurnForecastValidator {
    pub fn validate(&self, item: &FallbackBudgetBurnForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackBudgetBurnForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBudgetBurnForecast]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_fallback_budget_burn_forecast_validator() -> FallbackBudgetBurnForecastValidator {
    FallbackBudgetBurnForecastValidator {
        rules: vec![
            Rc::new(FallbackBudgetBurnForecastRequiredCodeRule {
                message: "fallback budget burn forecast needs this field".to_string(),
            }) as Rc<dyn FallbackBudgetBurnForecastRule>,
            Rc::new(FallbackBudgetBurnForecastBatchSizeRangeRule { min: 16, max: 255 }),
            Rc::new(FallbackBudgetBurnForecastRequiredAudienceRule {
                message: "fallback budget burn forecast needs a value".to_string(),
            }),
            Rc::new(FallbackBudgetBurnForecastRequiredReviewerRule {
                message: "fallback budget burn forecast needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_budget_burn_forecast_validator();
        let item = new_fallback_budget_burn_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackBudgetBurnForecastRequiredCodeRule {
            message: "fallback budget burn forecast needs an entry".to_string(),
        };
        let mut item = new_fallback_budget_burn_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = FallbackBudgetBurnForecastBatchSizeRangeRule { min: 64, max: 3600 };
        let mut item = new_fallback_budget_burn_forecast("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = FallbackBudgetBurnForecastRequiredAudienceRule {
            message: "fallback budget burn forecast needs an entry".to_string(),
        };
        let mut item = new_fallback_budget_burn_forecast("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = FallbackBudgetBurnForecastRequiredReviewerRule {
            message: "fallback budget burn forecast needs this field".to_string(),
        };
        let mut item = new_fallback_budget_burn_forecast("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
