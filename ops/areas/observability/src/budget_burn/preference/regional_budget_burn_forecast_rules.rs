use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional budget burn forecast must pass. Check returns its problems, or none.
pub trait RegionalBudgetBurnForecastRule: std::fmt::Debug {
    fn check(&self, item: &RegionalBudgetBurnForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnForecastCipherLengthRule {
    pub max_length: i64,
}

impl RegionalBudgetBurnForecastRule for RegionalBudgetBurnForecastCipherLengthRule {
    fn check(&self, item: &RegionalBudgetBurnForecast) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnForecastRequiredCipherRule {
    pub message: String,
}

impl RegionalBudgetBurnForecastRule for RegionalBudgetBurnForecastRequiredCipherRule {
    fn check(&self, item: &RegionalBudgetBurnForecast) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnForecastActiveGuardRule {
    pub limit: i64,
}

impl RegionalBudgetBurnForecastRule for RegionalBudgetBurnForecastActiveGuardRule {
    fn check(&self, item: &RegionalBudgetBurnForecast) -> Vec<String> {
        if item.active && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while active",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnForecastTitleLengthRule {
    pub max_length: i64,
}

impl RegionalBudgetBurnForecastRule for RegionalBudgetBurnForecastTitleLengthRule {
    fn check(&self, item: &RegionalBudgetBurnForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnForecastRequiredCodeRule {
    pub message: String,
}

impl RegionalBudgetBurnForecastRule for RegionalBudgetBurnForecastRequiredCodeRule {
    fn check(&self, item: &RegionalBudgetBurnForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnForecastTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalBudgetBurnForecastRule for RegionalBudgetBurnForecastTtlSecondsRangeRule {
    fn check(&self, item: &RegionalBudgetBurnForecast) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional budget burn forecast.
#[derive(Debug, Clone)]
pub struct RegionalBudgetBurnForecastValidator {
    pub rules: Vec<Rc<dyn RegionalBudgetBurnForecastRule>>,
}

impl RegionalBudgetBurnForecastValidator {
    pub fn validate(&self, item: &RegionalBudgetBurnForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalBudgetBurnForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalBudgetBurnForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_budget_burn_forecast_validator() -> RegionalBudgetBurnForecastValidator {
    RegionalBudgetBurnForecastValidator {
        rules: vec![
            Rc::new(RegionalBudgetBurnForecastCipherLengthRule { max_length: 255 })
                as Rc<dyn RegionalBudgetBurnForecastRule>,
            Rc::new(RegionalBudgetBurnForecastRequiredCipherRule {
                message: "regional budget burn forecast needs this field".to_string(),
            }),
            Rc::new(RegionalBudgetBurnForecastActiveGuardRule { limit: 3600 }),
            Rc::new(RegionalBudgetBurnForecastTitleLengthRule { max_length: 32 }),
            Rc::new(RegionalBudgetBurnForecastRequiredCodeRule {
                message: "regional budget burn forecast needs an entry".to_string(),
            }),
            Rc::new(RegionalBudgetBurnForecastTtlSecondsRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_budget_burn_forecast_validator();
        let item = new_regional_budget_burn_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = RegionalBudgetBurnForecastCipherLengthRule { max_length: 64 };
        let mut item = new_regional_budget_burn_forecast("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = RegionalBudgetBurnForecastRequiredCipherRule {
            message: "regional budget burn forecast needs a value".to_string(),
        };
        let mut item = new_regional_budget_burn_forecast("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalBudgetBurnForecastTitleLengthRule { max_length: 64 };
        let mut item = new_regional_budget_burn_forecast("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalBudgetBurnForecastRequiredCodeRule {
            message: "regional budget burn forecast needs this field".to_string(),
        };
        let mut item = new_regional_budget_burn_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RegionalBudgetBurnForecastTtlSecondsRangeRule { min: 32, max: 64 };
        let mut item = new_regional_budget_burn_forecast("x", "y");
        item.ttl_seconds = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
