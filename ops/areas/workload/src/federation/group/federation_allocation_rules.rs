use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation allocation must pass. Check returns its problems, or none.
pub trait FederationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &FederationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationAllocationRequiredNameRule {
    pub message: String,
}

impl FederationAllocationRule for FederationAllocationRequiredNameRule {
    fn check(&self, item: &FederationAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationAllocationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FederationAllocationRule for FederationAllocationAllowedNameRule {
    fn check(&self, item: &FederationAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationAllocationRequiredSegmentRule {
    pub message: String,
}

impl FederationAllocationRule for FederationAllocationRequiredSegmentRule {
    fn check(&self, item: &FederationAllocation) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation allocation.
#[derive(Debug, Clone)]
pub struct FederationAllocationValidator {
    pub rules: Vec<Rc<dyn FederationAllocationRule>>,
}

impl FederationAllocationValidator {
    pub fn validate(&self, item: &FederationAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_allocation_validator() -> FederationAllocationValidator {
    FederationAllocationValidator {
        rules: vec![
            Rc::new(FederationAllocationRequiredNameRule {
                message: "federation allocation needs this field".to_string(),
            }) as Rc<dyn FederationAllocationRule>,
            Rc::new(FederationAllocationAllowedNameRule {
                allowed: vec!["scheduled".to_string(), "batched".to_string()],
            }),
            Rc::new(FederationAllocationRequiredSegmentRule {
                message: "federation allocation needs a value".to_string(),
            }),
        ],
    }
}
