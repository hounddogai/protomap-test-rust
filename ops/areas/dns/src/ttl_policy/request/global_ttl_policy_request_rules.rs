use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global ttl policy request must pass. Check returns its problems, or none.
pub trait GlobalTtlPolicyRequestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTtlPolicyRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyRequestRequiredCodeRule {
    pub message: String,
}

impl GlobalTtlPolicyRequestRule for GlobalTtlPolicyRequestRequiredCodeRule {
    fn check(&self, item: &GlobalTtlPolicyRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyRequestDescriptionLengthRule {
    pub max_length: i64,
}

impl GlobalTtlPolicyRequestRule for GlobalTtlPolicyRequestDescriptionLengthRule {
    fn check(&self, item: &GlobalTtlPolicyRequest) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyRequestLabelLengthRule {
    pub max_length: i64,
}

impl GlobalTtlPolicyRequestRule for GlobalTtlPolicyRequestLabelLengthRule {
    fn check(&self, item: &GlobalTtlPolicyRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyRequestRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalTtlPolicyRequestRule for GlobalTtlPolicyRequestRetriesRangeRule {
    fn check(&self, item: &GlobalTtlPolicyRequest) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global ttl policy request.
#[derive(Debug, Clone)]
pub struct GlobalTtlPolicyRequestValidator {
    pub rules: Vec<Rc<dyn GlobalTtlPolicyRequestRule>>,
}

impl GlobalTtlPolicyRequestValidator {
    pub fn validate(&self, item: &GlobalTtlPolicyRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalTtlPolicyRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTtlPolicyRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_ttl_policy_request_validator() -> GlobalTtlPolicyRequestValidator {
    GlobalTtlPolicyRequestValidator {
        rules: vec![
            Rc::new(GlobalTtlPolicyRequestRequiredCodeRule {
                message: "global ttl policy request needs a value".to_string(),
            }) as Rc<dyn GlobalTtlPolicyRequestRule>,
            Rc::new(GlobalTtlPolicyRequestDescriptionLengthRule { max_length: 120 }),
            Rc::new(GlobalTtlPolicyRequestLabelLengthRule { max_length: 64 }),
            Rc::new(GlobalTtlPolicyRequestRetriesRangeRule { min: 120, max: 120 }),
        ],
    }
}
