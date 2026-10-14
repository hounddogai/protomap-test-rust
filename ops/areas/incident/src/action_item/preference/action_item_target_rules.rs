use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item target must pass. Check returns its problems, or none.
pub trait ActionItemTargetRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTargetRequiredCodeRule {
    pub message: String,
}

impl ActionItemTargetRule for ActionItemTargetRequiredCodeRule {
    fn check(&self, item: &ActionItemTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTargetCodePrefixRule {
    pub prefix: String,
}

impl ActionItemTargetRule for ActionItemTargetCodePrefixRule {
    fn check(&self, item: &ActionItemTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTargetExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemTargetRule for ActionItemTargetExpiresAtRangeRule {
    fn check(&self, item: &ActionItemTarget) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTargetSegmentLengthRule {
    pub max_length: i64,
}

impl ActionItemTargetRule for ActionItemTargetSegmentLengthRule {
    fn check(&self, item: &ActionItemTarget) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item target.
#[derive(Debug, Clone)]
pub struct ActionItemTargetValidator {
    pub rules: Vec<Rc<dyn ActionItemTargetRule>>,
}

impl ActionItemTargetValidator {
    pub fn validate(&self, item: &ActionItemTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_target_validator() -> ActionItemTargetValidator {
    ActionItemTargetValidator {
        rules: vec![
            Rc::new(ActionItemTargetRequiredCodeRule {
                message: "action item target needs this field".to_string(),
            }) as Rc<dyn ActionItemTargetRule>,
            Rc::new(ActionItemTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ActionItemTargetExpiresAtRangeRule { min: 16, max: 16 }),
            Rc::new(ActionItemTargetSegmentLengthRule { max_length: 32 }),
        ],
    }
}
