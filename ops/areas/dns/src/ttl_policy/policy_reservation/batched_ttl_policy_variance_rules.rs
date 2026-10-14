use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched ttl policy variance must pass. Check returns its problems, or none.
pub trait BatchedTtlPolicyVarianceRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTtlPolicyVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyVarianceAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BatchedTtlPolicyVarianceRule for BatchedTtlPolicyVarianceAllowedNameRule {
    fn check(&self, item: &BatchedTtlPolicyVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyVarianceCodeLengthRule {
    pub max_length: i64,
}

impl BatchedTtlPolicyVarianceRule for BatchedTtlPolicyVarianceCodeLengthRule {
    fn check(&self, item: &BatchedTtlPolicyVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyVarianceCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedTtlPolicyVarianceRule for BatchedTtlPolicyVarianceCostRangeRule {
    fn check(&self, item: &BatchedTtlPolicyVariance) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyVarianceCodePrefixRule {
    pub prefix: String,
}

impl BatchedTtlPolicyVarianceRule for BatchedTtlPolicyVarianceCodePrefixRule {
    fn check(&self, item: &BatchedTtlPolicyVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched ttl policy variance.
#[derive(Debug, Clone)]
pub struct BatchedTtlPolicyVarianceValidator {
    pub rules: Vec<Rc<dyn BatchedTtlPolicyVarianceRule>>,
}

impl BatchedTtlPolicyVarianceValidator {
    pub fn validate(&self, item: &BatchedTtlPolicyVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedTtlPolicyVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTtlPolicyVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_ttl_policy_variance_validator() -> BatchedTtlPolicyVarianceValidator {
    BatchedTtlPolicyVarianceValidator {
        rules: vec![
            Rc::new(BatchedTtlPolicyVarianceAllowedNameRule { allowed: Vec::new() })
                as Rc<dyn BatchedTtlPolicyVarianceRule>,
            Rc::new(BatchedTtlPolicyVarianceCodeLengthRule { max_length: 80 }),
            Rc::new(BatchedTtlPolicyVarianceCostRangeRule { min: 1.7, max: 2769.1 }),
            Rc::new(BatchedTtlPolicyVarianceCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
