use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast estimate must pass. Check returns its problems, or none.
pub trait ForecastEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ForecastEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEstimateRegionLengthRule {
    pub max_length: i64,
}

impl ForecastEstimateRule for ForecastEstimateRegionLengthRule {
    fn check(&self, item: &ForecastEstimate) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEstimateRequiredTitleRule {
    pub message: String,
}

impl ForecastEstimateRule for ForecastEstimateRequiredTitleRule {
    fn check(&self, item: &ForecastEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEstimateCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastEstimateRule for ForecastEstimateCapacityRangeRule {
    fn check(&self, item: &ForecastEstimate) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEstimateCodePrefixRule {
    pub prefix: String,
}

impl ForecastEstimateRule for ForecastEstimateCodePrefixRule {
    fn check(&self, item: &ForecastEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastEstimateCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastEstimateRule for ForecastEstimateCreatedAtRangeRule {
    fn check(&self, item: &ForecastEstimate) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a forecast estimate.
#[derive(Debug, Clone)]
pub struct ForecastEstimateValidator {
    pub rules: Vec<Rc<dyn ForecastEstimateRule>>,
}

impl ForecastEstimateValidator {
    pub fn validate(&self, item: &ForecastEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_estimate_validator() -> ForecastEstimateValidator {
    ForecastEstimateValidator {
        rules: vec![
            Rc::new(ForecastEstimateRegionLengthRule { max_length: 16 }) as Rc<dyn ForecastEstimateRule>,
            Rc::new(ForecastEstimateRequiredTitleRule {
                message: "forecast estimate needs an entry".to_string(),
            }),
            Rc::new(ForecastEstimateCapacityRangeRule { min: 64, max: 120 }),
            Rc::new(ForecastEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ForecastEstimateCreatedAtRangeRule { min: 120, max: 255 }),
        ],
    }
}
