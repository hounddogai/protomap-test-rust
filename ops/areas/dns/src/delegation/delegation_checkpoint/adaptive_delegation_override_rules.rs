use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive delegation override must pass. Check returns its problems, or none.
pub trait AdaptiveDelegationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveDelegationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationOverrideReviewerLengthRule {
    pub max_length: i64,
}

impl AdaptiveDelegationOverrideRule for AdaptiveDelegationOverrideReviewerLengthRule {
    fn check(&self, item: &AdaptiveDelegationOverride) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationOverrideRequiredNameRule {
    pub message: String,
}

impl AdaptiveDelegationOverrideRule for AdaptiveDelegationOverrideRequiredNameRule {
    fn check(&self, item: &AdaptiveDelegationOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationOverrideRequiredCodeRule {
    pub message: String,
}

impl AdaptiveDelegationOverrideRule for AdaptiveDelegationOverrideRequiredCodeRule {
    fn check(&self, item: &AdaptiveDelegationOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive delegation override.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationOverrideValidator {
    pub rules: Vec<Rc<dyn AdaptiveDelegationOverrideRule>>,
}

impl AdaptiveDelegationOverrideValidator {
    pub fn validate(&self, item: &AdaptiveDelegationOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveDelegationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveDelegationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_delegation_override_validator() -> AdaptiveDelegationOverrideValidator {
    AdaptiveDelegationOverrideValidator {
        rules: vec![
            Rc::new(AdaptiveDelegationOverrideReviewerLengthRule { max_length: 3600 })
                as Rc<dyn AdaptiveDelegationOverrideRule>,
            Rc::new(AdaptiveDelegationOverrideRequiredNameRule {
                message: "adaptive delegation override needs a value".to_string(),
            }),
            Rc::new(AdaptiveDelegationOverrideRequiredCodeRule {
                message: "adaptive delegation override needs an entry".to_string(),
            }),
        ],
    }
}
