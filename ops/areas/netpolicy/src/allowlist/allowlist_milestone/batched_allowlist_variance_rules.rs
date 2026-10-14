use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched allowlist variance must pass. Check returns its problems, or none.
pub trait BatchedAllowlistVarianceRule: std::fmt::Debug {
    fn check(&self, item: &BatchedAllowlistVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistVarianceRequiredSourceRule {
    pub message: String,
}

impl BatchedAllowlistVarianceRule for BatchedAllowlistVarianceRequiredSourceRule {
    fn check(&self, item: &BatchedAllowlistVariance) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistVarianceSourceLengthRule {
    pub max_length: i64,
}

impl BatchedAllowlistVarianceRule for BatchedAllowlistVarianceSourceLengthRule {
    fn check(&self, item: &BatchedAllowlistVariance) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistVarianceMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedAllowlistVarianceRule for BatchedAllowlistVarianceMarginRangeRule {
    fn check(&self, item: &BatchedAllowlistVariance) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched allowlist variance.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistVarianceValidator {
    pub rules: Vec<Rc<dyn BatchedAllowlistVarianceRule>>,
}

impl BatchedAllowlistVarianceValidator {
    pub fn validate(&self, item: &BatchedAllowlistVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedAllowlistVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedAllowlistVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_allowlist_variance_validator() -> BatchedAllowlistVarianceValidator {
    BatchedAllowlistVarianceValidator {
        rules: vec![
            Rc::new(BatchedAllowlistVarianceRequiredSourceRule {
                message: "batched allowlist variance needs an entry".to_string(),
            }) as Rc<dyn BatchedAllowlistVarianceRule>,
            Rc::new(BatchedAllowlistVarianceSourceLengthRule { max_length: 80 }),
            Rc::new(BatchedAllowlistVarianceMarginRangeRule { min: 5.3, max: 1671.4 }),
        ],
    }
}
