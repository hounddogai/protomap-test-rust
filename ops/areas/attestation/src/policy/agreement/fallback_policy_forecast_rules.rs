use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback policy forecast must pass. Check returns its problems, or none.
pub trait FallbackPolicyForecastRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPolicyForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyForecastDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPolicyForecastRule for FallbackPolicyForecastDurationMinutesRangeRule {
    fn check(&self, item: &FallbackPolicyForecast) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyForecastRequiredCodeRule {
    pub message: String,
}

impl FallbackPolicyForecastRule for FallbackPolicyForecastRequiredCodeRule {
    fn check(&self, item: &FallbackPolicyForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyForecastQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPolicyForecastRule for FallbackPolicyForecastQuantityRangeRule {
    fn check(&self, item: &FallbackPolicyForecast) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyForecastNameLengthRule {
    pub max_length: i64,
}

impl FallbackPolicyForecastRule for FallbackPolicyForecastNameLengthRule {
    fn check(&self, item: &FallbackPolicyForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyForecastRequiredRegionRule {
    pub message: String,
}

impl FallbackPolicyForecastRule for FallbackPolicyForecastRequiredRegionRule {
    fn check(&self, item: &FallbackPolicyForecast) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback policy forecast.
#[derive(Debug, Clone)]
pub struct FallbackPolicyForecastValidator {
    pub rules: Vec<Rc<dyn FallbackPolicyForecastRule>>,
}

impl FallbackPolicyForecastValidator {
    pub fn validate(&self, item: &FallbackPolicyForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackPolicyForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPolicyForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_policy_forecast_validator() -> FallbackPolicyForecastValidator {
    FallbackPolicyForecastValidator {
        rules: vec![
            Rc::new(FallbackPolicyForecastDurationMinutesRangeRule { min: 255, max: 1000 })
                as Rc<dyn FallbackPolicyForecastRule>,
            Rc::new(FallbackPolicyForecastRequiredCodeRule {
                message: "fallback policy forecast needs this field".to_string(),
            }),
            Rc::new(FallbackPolicyForecastQuantityRangeRule { min: 16, max: 64 }),
            Rc::new(FallbackPolicyForecastNameLengthRule { max_length: 255 }),
            Rc::new(FallbackPolicyForecastRequiredRegionRule {
                message: "fallback policy forecast needs an entry".to_string(),
            }),
        ],
    }
}
