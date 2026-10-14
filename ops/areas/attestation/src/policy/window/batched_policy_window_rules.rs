use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched policy window must pass. Check returns its problems, or none.
pub trait BatchedPolicyWindowRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicyWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyWindowRequiredCodeRule {
    pub message: String,
}

impl BatchedPolicyWindowRule for BatchedPolicyWindowRequiredCodeRule {
    fn check(&self, item: &BatchedPolicyWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyWindowCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPolicyWindowRule for BatchedPolicyWindowCreatedAtRangeRule {
    fn check(&self, item: &BatchedPolicyWindow) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyWindowTitleLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyWindowRule for BatchedPolicyWindowTitleLengthRule {
    fn check(&self, item: &BatchedPolicyWindow) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyWindowRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPolicyWindowRule for BatchedPolicyWindowRotationDaysRangeRule {
    fn check(&self, item: &BatchedPolicyWindow) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched policy window.
#[derive(Debug, Clone)]
pub struct BatchedPolicyWindowValidator {
    pub rules: Vec<Rc<dyn BatchedPolicyWindowRule>>,
}

impl BatchedPolicyWindowValidator {
    pub fn validate(&self, item: &BatchedPolicyWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPolicyWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicyWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_window_validator() -> BatchedPolicyWindowValidator {
    BatchedPolicyWindowValidator {
        rules: vec![
            Rc::new(BatchedPolicyWindowRequiredCodeRule {
                message: "batched policy window needs a value".to_string(),
            }) as Rc<dyn BatchedPolicyWindowRule>,
            Rc::new(BatchedPolicyWindowCreatedAtRangeRule { min: 120, max: 3600 }),
            Rc::new(BatchedPolicyWindowTitleLengthRule { max_length: 255 }),
            Rc::new(BatchedPolicyWindowRotationDaysRangeRule { min: 32, max: 120 }),
        ],
    }
}
