use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout estimate must pass. Check returns its problems, or none.
pub trait RolloutEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RolloutEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimateCodePrefixRule {
    pub prefix: String,
}

impl RolloutEstimateRule for RolloutEstimateCodePrefixRule {
    fn check(&self, item: &RolloutEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimateRequiredCodeRule {
    pub message: String,
}

impl RolloutEstimateRule for RolloutEstimateRequiredCodeRule {
    fn check(&self, item: &RolloutEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimateRequiredNotesRule {
    pub message: String,
}

impl RolloutEstimateRule for RolloutEstimateRequiredNotesRule {
    fn check(&self, item: &RolloutEstimate) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimateNameLengthRule {
    pub max_length: i64,
}

impl RolloutEstimateRule for RolloutEstimateNameLengthRule {
    fn check(&self, item: &RolloutEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimateRequiredNameRule {
    pub message: String,
}

impl RolloutEstimateRule for RolloutEstimateRequiredNameRule {
    fn check(&self, item: &RolloutEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout estimate.
#[derive(Debug, Clone)]
pub struct RolloutEstimateValidator {
    pub rules: Vec<Rc<dyn RolloutEstimateRule>>,
}

impl RolloutEstimateValidator {
    pub fn validate(&self, item: &RolloutEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_estimate_validator() -> RolloutEstimateValidator {
    RolloutEstimateValidator {
        rules: vec![
            Rc::new(RolloutEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RolloutEstimateRule>,
            Rc::new(RolloutEstimateRequiredCodeRule {
                message: "rollout estimate needs an entry".to_string(),
            }),
            Rc::new(RolloutEstimateRequiredNotesRule {
                message: "rollout estimate needs this field".to_string(),
            }),
            Rc::new(RolloutEstimateNameLengthRule { max_length: 1000 }),
            Rc::new(RolloutEstimateRequiredNameRule {
                message: "rollout estimate needs an entry".to_string(),
            }),
        ],
    }
}
