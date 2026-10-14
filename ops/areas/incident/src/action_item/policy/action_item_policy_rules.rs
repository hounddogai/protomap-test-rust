use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item policy must pass. Check returns its problems, or none.
pub trait ActionItemPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPolicyBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemPolicyRule for ActionItemPolicyBudgetRangeRule {
    fn check(&self, item: &ActionItemPolicy) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPolicyScopeLengthRule {
    pub max_length: i64,
}

impl ActionItemPolicyRule for ActionItemPolicyScopeLengthRule {
    fn check(&self, item: &ActionItemPolicy) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPolicyNameLengthRule {
    pub max_length: i64,
}

impl ActionItemPolicyRule for ActionItemPolicyNameLengthRule {
    fn check(&self, item: &ActionItemPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPolicyRequiredCodeRule {
    pub message: String,
}

impl ActionItemPolicyRule for ActionItemPolicyRequiredCodeRule {
    fn check(&self, item: &ActionItemPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPolicyCodePrefixRule {
    pub prefix: String,
}

impl ActionItemPolicyRule for ActionItemPolicyCodePrefixRule {
    fn check(&self, item: &ActionItemPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item policy.
#[derive(Debug, Clone)]
pub struct ActionItemPolicyValidator {
    pub rules: Vec<Rc<dyn ActionItemPolicyRule>>,
}

impl ActionItemPolicyValidator {
    pub fn validate(&self, item: &ActionItemPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_policy_validator() -> ActionItemPolicyValidator {
    ActionItemPolicyValidator {
        rules: vec![
            Rc::new(ActionItemPolicyBudgetRangeRule { min: 1.0, max: 2577.5 }) as Rc<dyn ActionItemPolicyRule>,
            Rc::new(ActionItemPolicyScopeLengthRule { max_length: 255 }),
            Rc::new(ActionItemPolicyNameLengthRule { max_length: 1000 }),
            Rc::new(ActionItemPolicyRequiredCodeRule {
                message: "action item policy needs a value".to_string(),
            }),
            Rc::new(ActionItemPolicyCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
