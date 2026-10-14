use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary delegation assignment must pass. Check returns its problems, or none.
pub trait PrimaryDelegationAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDelegationAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationAssignmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryDelegationAssignmentRule for PrimaryDelegationAssignmentDiscountRangeRule {
    fn check(&self, item: &PrimaryDelegationAssignment) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationAssignmentClusterLengthRule {
    pub max_length: i64,
}

impl PrimaryDelegationAssignmentRule for PrimaryDelegationAssignmentClusterLengthRule {
    fn check(&self, item: &PrimaryDelegationAssignment) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationAssignmentCodePrefixRule {
    pub prefix: String,
}

impl PrimaryDelegationAssignmentRule for PrimaryDelegationAssignmentCodePrefixRule {
    fn check(&self, item: &PrimaryDelegationAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationAssignmentRequiredClusterRule {
    pub message: String,
}

impl PrimaryDelegationAssignmentRule for PrimaryDelegationAssignmentRequiredClusterRule {
    fn check(&self, item: &PrimaryDelegationAssignment) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationAssignmentRequiredLabelRule {
    pub message: String,
}

impl PrimaryDelegationAssignmentRule for PrimaryDelegationAssignmentRequiredLabelRule {
    fn check(&self, item: &PrimaryDelegationAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary delegation assignment.
#[derive(Debug, Clone)]
pub struct PrimaryDelegationAssignmentValidator {
    pub rules: Vec<Rc<dyn PrimaryDelegationAssignmentRule>>,
}

impl PrimaryDelegationAssignmentValidator {
    pub fn validate(&self, item: &PrimaryDelegationAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryDelegationAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDelegationAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_delegation_assignment_validator() -> PrimaryDelegationAssignmentValidator {
    PrimaryDelegationAssignmentValidator {
        rules: vec![
            Rc::new(PrimaryDelegationAssignmentDiscountRangeRule { min: 2.4, max: 2780.9 })
                as Rc<dyn PrimaryDelegationAssignmentRule>,
            Rc::new(PrimaryDelegationAssignmentClusterLengthRule { max_length: 16 }),
            Rc::new(PrimaryDelegationAssignmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryDelegationAssignmentRequiredClusterRule {
                message: "primary delegation assignment needs this field".to_string(),
            }),
            Rc::new(PrimaryDelegationAssignmentRequiredLabelRule {
                message: "primary delegation assignment needs an entry".to_string(),
            }),
        ],
    }
}
