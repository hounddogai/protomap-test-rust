use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout threshold must pass. Check returns its problems, or none.
pub trait RolloutThresholdRule: std::fmt::Debug {
    fn check(&self, item: &RolloutThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutThresholdRequiredCodeRule {
    pub message: String,
}

impl RolloutThresholdRule for RolloutThresholdRequiredCodeRule {
    fn check(&self, item: &RolloutThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutThresholdCountryLengthRule {
    pub max_length: i64,
}

impl RolloutThresholdRule for RolloutThresholdCountryLengthRule {
    fn check(&self, item: &RolloutThreshold) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutThresholdCodeLengthRule {
    pub max_length: i64,
}

impl RolloutThresholdRule for RolloutThresholdCodeLengthRule {
    fn check(&self, item: &RolloutThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutThresholdRequiredCountryRule {
    pub message: String,
}

impl RolloutThresholdRule for RolloutThresholdRequiredCountryRule {
    fn check(&self, item: &RolloutThreshold) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout threshold.
#[derive(Debug, Clone)]
pub struct RolloutThresholdValidator {
    pub rules: Vec<Rc<dyn RolloutThresholdRule>>,
}

impl RolloutThresholdValidator {
    pub fn validate(&self, item: &RolloutThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_threshold_validator() -> RolloutThresholdValidator {
    RolloutThresholdValidator {
        rules: vec![
            Rc::new(RolloutThresholdRequiredCodeRule {
                message: "rollout threshold needs a value".to_string(),
            }) as Rc<dyn RolloutThresholdRule>,
            Rc::new(RolloutThresholdCountryLengthRule { max_length: 86400 }),
            Rc::new(RolloutThresholdCodeLengthRule { max_length: 80 }),
            Rc::new(RolloutThresholdRequiredCountryRule {
                message: "rollout threshold needs an entry".to_string(),
            }),
        ],
    }
}
