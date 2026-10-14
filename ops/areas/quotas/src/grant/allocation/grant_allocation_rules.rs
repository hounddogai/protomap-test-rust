use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant allocation must pass. Check returns its problems, or none.
pub trait GrantAllocationRule: std::fmt::Debug {
    fn check(&self, item: &GrantAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAllocationRequiredOwnerIdRule {
    pub message: String,
}

impl GrantAllocationRule for GrantAllocationRequiredOwnerIdRule {
    fn check(&self, item: &GrantAllocation) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAllocationLabelLengthRule {
    pub max_length: i64,
}

impl GrantAllocationRule for GrantAllocationLabelLengthRule {
    fn check(&self, item: &GrantAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAllocationRequiredFingerprintRule {
    pub message: String,
}

impl GrantAllocationRule for GrantAllocationRequiredFingerprintRule {
    fn check(&self, item: &GrantAllocation) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant allocation.
#[derive(Debug, Clone)]
pub struct GrantAllocationValidator {
    pub rules: Vec<Rc<dyn GrantAllocationRule>>,
}

impl GrantAllocationValidator {
    pub fn validate(&self, item: &GrantAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_allocation_validator() -> GrantAllocationValidator {
    GrantAllocationValidator {
        rules: vec![
            Rc::new(GrantAllocationRequiredOwnerIdRule {
                message: "grant allocation needs a value".to_string(),
            }) as Rc<dyn GrantAllocationRule>,
            Rc::new(GrantAllocationLabelLengthRule { max_length: 3600 }),
            Rc::new(GrantAllocationRequiredFingerprintRule {
                message: "grant allocation needs an entry".to_string(),
            }),
        ],
    }
}
