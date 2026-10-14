use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict policy forecast must pass. Check returns its problems, or none.
pub trait StrictPolicyForecastRule: std::fmt::Debug {
    fn check(&self, item: &StrictPolicyForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyForecastRequiredCodeRule {
    pub message: String,
}

impl StrictPolicyForecastRule for StrictPolicyForecastRequiredCodeRule {
    fn check(&self, item: &StrictPolicyForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyForecastActiveGuardRule {
    pub limit: i64,
}

impl StrictPolicyForecastRule for StrictPolicyForecastActiveGuardRule {
    fn check(&self, item: &StrictPolicyForecast) -> Vec<String> {
        if item.active && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while active",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyForecastQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPolicyForecastRule for StrictPolicyForecastQuantityRangeRule {
    fn check(&self, item: &StrictPolicyForecast) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict policy forecast.
#[derive(Debug, Clone)]
pub struct StrictPolicyForecastValidator {
    pub rules: Vec<Rc<dyn StrictPolicyForecastRule>>,
}

impl StrictPolicyForecastValidator {
    pub fn validate(&self, item: &StrictPolicyForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictPolicyForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPolicyForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_policy_forecast_validator() -> StrictPolicyForecastValidator {
    StrictPolicyForecastValidator {
        rules: vec![
            Rc::new(StrictPolicyForecastRequiredCodeRule {
                message: "strict policy forecast needs this field".to_string(),
            }) as Rc<dyn StrictPolicyForecastRule>,
            Rc::new(StrictPolicyForecastActiveGuardRule { limit: 16 }),
            Rc::new(StrictPolicyForecastQuantityRangeRule { min: 80, max: 1000 }),
        ],
    }
}
