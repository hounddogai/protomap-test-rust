use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling federation allocation must pass. Check returns its problems, or none.
pub trait RollingFederationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &RollingFederationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocationRequiredFingerprintRule {
    pub message: String,
}

impl RollingFederationAllocationRule for RollingFederationAllocationRequiredFingerprintRule {
    fn check(&self, item: &RollingFederationAllocation) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocationLabelLengthRule {
    pub max_length: i64,
}

impl RollingFederationAllocationRule for RollingFederationAllocationLabelLengthRule {
    fn check(&self, item: &RollingFederationAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocationRequiredCodeRule {
    pub message: String,
}

impl RollingFederationAllocationRule for RollingFederationAllocationRequiredCodeRule {
    fn check(&self, item: &RollingFederationAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocationCodeLengthRule {
    pub max_length: i64,
}

impl RollingFederationAllocationRule for RollingFederationAllocationCodeLengthRule {
    fn check(&self, item: &RollingFederationAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocationFingerprintLengthRule {
    pub max_length: i64,
}

impl RollingFederationAllocationRule for RollingFederationAllocationFingerprintLengthRule {
    fn check(&self, item: &RollingFederationAllocation) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocationRequiredLabelRule {
    pub message: String,
}

impl RollingFederationAllocationRule for RollingFederationAllocationRequiredLabelRule {
    fn check(&self, item: &RollingFederationAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling federation allocation.
#[derive(Debug, Clone)]
pub struct RollingFederationAllocationValidator {
    pub rules: Vec<Rc<dyn RollingFederationAllocationRule>>,
}

impl RollingFederationAllocationValidator {
    pub fn validate(&self, item: &RollingFederationAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingFederationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingFederationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_federation_allocation_validator() -> RollingFederationAllocationValidator {
    RollingFederationAllocationValidator {
        rules: vec![
            Rc::new(RollingFederationAllocationRequiredFingerprintRule {
                message: "rolling federation allocation needs this field".to_string(),
            }) as Rc<dyn RollingFederationAllocationRule>,
            Rc::new(RollingFederationAllocationLabelLengthRule { max_length: 16 }),
            Rc::new(RollingFederationAllocationRequiredCodeRule {
                message: "rolling federation allocation needs a value".to_string(),
            }),
            Rc::new(RollingFederationAllocationCodeLengthRule { max_length: 64 }),
            Rc::new(RollingFederationAllocationFingerprintLengthRule { max_length: 64 }),
            Rc::new(RollingFederationAllocationRequiredLabelRule {
                message: "rolling federation allocation needs a value".to_string(),
            }),
        ],
    }
}
