use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched policy baseline must pass. Check returns its problems, or none.
pub trait BatchedPolicyBaselineRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicyBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyBaselineRequiredLocaleRule {
    pub message: String,
}

impl BatchedPolicyBaselineRule for BatchedPolicyBaselineRequiredLocaleRule {
    fn check(&self, item: &BatchedPolicyBaseline) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyBaselineRequiredLabelRule {
    pub message: String,
}

impl BatchedPolicyBaselineRule for BatchedPolicyBaselineRequiredLabelRule {
    fn check(&self, item: &BatchedPolicyBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyBaselineRequiredCategoryRule {
    pub message: String,
}

impl BatchedPolicyBaselineRule for BatchedPolicyBaselineRequiredCategoryRule {
    fn check(&self, item: &BatchedPolicyBaseline) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyBaselineCategoryLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyBaselineRule for BatchedPolicyBaselineCategoryLengthRule {
    fn check(&self, item: &BatchedPolicyBaseline) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched policy baseline.
#[derive(Debug, Clone)]
pub struct BatchedPolicyBaselineValidator {
    pub rules: Vec<Rc<dyn BatchedPolicyBaselineRule>>,
}

impl BatchedPolicyBaselineValidator {
    pub fn validate(&self, item: &BatchedPolicyBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPolicyBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicyBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_baseline_validator() -> BatchedPolicyBaselineValidator {
    BatchedPolicyBaselineValidator {
        rules: vec![
            Rc::new(BatchedPolicyBaselineRequiredLocaleRule {
                message: "batched policy baseline needs a value".to_string(),
            }) as Rc<dyn BatchedPolicyBaselineRule>,
            Rc::new(BatchedPolicyBaselineRequiredLabelRule {
                message: "batched policy baseline needs a value".to_string(),
            }),
            Rc::new(BatchedPolicyBaselineRequiredCategoryRule {
                message: "batched policy baseline needs an entry".to_string(),
            }),
            Rc::new(BatchedPolicyBaselineCategoryLengthRule { max_length: 255 }),
        ],
    }
}
