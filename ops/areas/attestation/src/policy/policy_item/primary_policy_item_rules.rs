use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary policy item must pass. Check returns its problems, or none.
pub trait PrimaryPolicyItemRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPolicyItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPolicyItemRequiredOwnerIdRule {
    pub message: String,
}

impl PrimaryPolicyItemRule for PrimaryPolicyItemRequiredOwnerIdRule {
    fn check(&self, item: &PrimaryPolicyItem) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPolicyItemCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryPolicyItemRule for PrimaryPolicyItemCodeLengthRule {
    fn check(&self, item: &PrimaryPolicyItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPolicyItemNameLengthRule {
    pub max_length: i64,
}

impl PrimaryPolicyItemRule for PrimaryPolicyItemNameLengthRule {
    fn check(&self, item: &PrimaryPolicyItem) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary policy item.
#[derive(Debug, Clone)]
pub struct PrimaryPolicyItemValidator {
    pub rules: Vec<Rc<dyn PrimaryPolicyItemRule>>,
}

impl PrimaryPolicyItemValidator {
    pub fn validate(&self, item: &PrimaryPolicyItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryPolicyItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPolicyItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_policy_item_validator() -> PrimaryPolicyItemValidator {
    PrimaryPolicyItemValidator {
        rules: vec![
            Rc::new(PrimaryPolicyItemRequiredOwnerIdRule {
                message: "primary policy item needs an entry".to_string(),
            }) as Rc<dyn PrimaryPolicyItemRule>,
            Rc::new(PrimaryPolicyItemCodeLengthRule { max_length: 32 }),
            Rc::new(PrimaryPolicyItemNameLengthRule { max_length: 3600 }),
        ],
    }
}
