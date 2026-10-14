use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence allocation must pass. Check returns its problems, or none.
pub trait EvidenceAllocationRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAllocationReviewerLengthRule {
    pub max_length: i64,
}

impl EvidenceAllocationRule for EvidenceAllocationReviewerLengthRule {
    fn check(&self, item: &EvidenceAllocation) -> Vec<String> {
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
pub struct EvidenceAllocationLabelLengthRule {
    pub max_length: i64,
}

impl EvidenceAllocationRule for EvidenceAllocationLabelLengthRule {
    fn check(&self, item: &EvidenceAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAllocationCodePrefixRule {
    pub prefix: String,
}

impl EvidenceAllocationRule for EvidenceAllocationCodePrefixRule {
    fn check(&self, item: &EvidenceAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence allocation.
#[derive(Debug, Clone)]
pub struct EvidenceAllocationValidator {
    pub rules: Vec<Rc<dyn EvidenceAllocationRule>>,
}

impl EvidenceAllocationValidator {
    pub fn validate(&self, item: &EvidenceAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_allocation_validator() -> EvidenceAllocationValidator {
    EvidenceAllocationValidator {
        rules: vec![
            Rc::new(EvidenceAllocationReviewerLengthRule { max_length: 3600 }) as Rc<dyn EvidenceAllocationRule>,
            Rc::new(EvidenceAllocationLabelLengthRule { max_length: 16 }),
            Rc::new(EvidenceAllocationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
