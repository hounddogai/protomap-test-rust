use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched forecast cycle must pass. Check returns its problems, or none.
pub trait BatchedForecastCycleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedForecastCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastCycleRequiredLocaleRule {
    pub message: String,
}

impl BatchedForecastCycleRule for BatchedForecastCycleRequiredLocaleRule {
    fn check(&self, item: &BatchedForecastCycle) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastCycleRequiredLabelRule {
    pub message: String,
}

impl BatchedForecastCycleRule for BatchedForecastCycleRequiredLabelRule {
    fn check(&self, item: &BatchedForecastCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastCycleLocaleLengthRule {
    pub max_length: i64,
}

impl BatchedForecastCycleRule for BatchedForecastCycleLocaleLengthRule {
    fn check(&self, item: &BatchedForecastCycle) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastCycleLabelLengthRule {
    pub max_length: i64,
}

impl BatchedForecastCycleRule for BatchedForecastCycleLabelLengthRule {
    fn check(&self, item: &BatchedForecastCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched forecast cycle.
#[derive(Debug, Clone)]
pub struct BatchedForecastCycleValidator {
    pub rules: Vec<Rc<dyn BatchedForecastCycleRule>>,
}

impl BatchedForecastCycleValidator {
    pub fn validate(&self, item: &BatchedForecastCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedForecastCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedForecastCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_forecast_cycle_validator() -> BatchedForecastCycleValidator {
    BatchedForecastCycleValidator {
        rules: vec![
            Rc::new(BatchedForecastCycleRequiredLocaleRule {
                message: "batched forecast cycle needs an entry".to_string(),
            }) as Rc<dyn BatchedForecastCycleRule>,
            Rc::new(BatchedForecastCycleRequiredLabelRule {
                message: "batched forecast cycle needs a value".to_string(),
            }),
            Rc::new(BatchedForecastCycleLocaleLengthRule { max_length: 3600 }),
            Rc::new(BatchedForecastCycleLabelLengthRule { max_length: 120 }),
        ],
    }
}
