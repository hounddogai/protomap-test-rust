use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast override must pass. Check returns its problems, or none.
pub trait ForecastOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ForecastOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastOverrideCodePrefixRule {
    pub prefix: String,
}

impl ForecastOverrideRule for ForecastOverrideCodePrefixRule {
    fn check(&self, item: &ForecastOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastOverrideCodeLengthRule {
    pub max_length: i64,
}

impl ForecastOverrideRule for ForecastOverrideCodeLengthRule {
    fn check(&self, item: &ForecastOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastOverrideTitleLengthRule {
    pub max_length: i64,
}

impl ForecastOverrideRule for ForecastOverrideTitleLengthRule {
    fn check(&self, item: &ForecastOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastOverrideRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastOverrideRule for ForecastOverrideRateRangeRule {
    fn check(&self, item: &ForecastOverride) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastOverrideActiveGuardRule {
    pub limit: f64,
}

impl ForecastOverrideRule for ForecastOverrideActiveGuardRule {
    fn check(&self, item: &ForecastOverride) -> Vec<String> {
        if item.active && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while active", item.rate, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast override.
#[derive(Debug, Clone)]
pub struct ForecastOverrideValidator {
    pub rules: Vec<Rc<dyn ForecastOverrideRule>>,
}

impl ForecastOverrideValidator {
    pub fn validate(&self, item: &ForecastOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_override_validator() -> ForecastOverrideValidator {
    ForecastOverrideValidator {
        rules: vec![
            Rc::new(ForecastOverrideCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn ForecastOverrideRule>,
            Rc::new(ForecastOverrideCodeLengthRule { max_length: 64 }),
            Rc::new(ForecastOverrideTitleLengthRule { max_length: 80 }),
            Rc::new(ForecastOverrideRateRangeRule { min: 8.9, max: 376.0 }),
            Rc::new(ForecastOverrideActiveGuardRule { limit: 1.0 }),
        ],
    }
}
