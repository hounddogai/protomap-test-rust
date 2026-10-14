use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global release policy must pass. Check returns its problems, or none.
pub trait GlobalReleasePolicyRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReleasePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleasePolicyQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalReleasePolicyRule for GlobalReleasePolicyQuantityRangeRule {
    fn check(&self, item: &GlobalReleasePolicy) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleasePolicyRequiredCodeRule {
    pub message: String,
}

impl GlobalReleasePolicyRule for GlobalReleasePolicyRequiredCodeRule {
    fn check(&self, item: &GlobalReleasePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleasePolicyKeyIdLengthRule {
    pub max_length: i64,
}

impl GlobalReleasePolicyRule for GlobalReleasePolicyKeyIdLengthRule {
    fn check(&self, item: &GlobalReleasePolicy) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleasePolicyCodeLengthRule {
    pub max_length: i64,
}

impl GlobalReleasePolicyRule for GlobalReleasePolicyCodeLengthRule {
    fn check(&self, item: &GlobalReleasePolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global release policy.
#[derive(Debug, Clone)]
pub struct GlobalReleasePolicyValidator {
    pub rules: Vec<Rc<dyn GlobalReleasePolicyRule>>,
}

impl GlobalReleasePolicyValidator {
    pub fn validate(&self, item: &GlobalReleasePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalReleasePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReleasePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_release_policy_validator() -> GlobalReleasePolicyValidator {
    GlobalReleasePolicyValidator {
        rules: vec![
            Rc::new(GlobalReleasePolicyQuantityRangeRule { min: 16, max: 80 }) as Rc<dyn GlobalReleasePolicyRule>,
            Rc::new(GlobalReleasePolicyRequiredCodeRule {
                message: "global release policy needs a value".to_string(),
            }),
            Rc::new(GlobalReleasePolicyKeyIdLengthRule { max_length: 86400 }),
            Rc::new(GlobalReleasePolicyCodeLengthRule { max_length: 255 }),
        ],
    }
}
