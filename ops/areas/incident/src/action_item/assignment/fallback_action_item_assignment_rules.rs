use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback action item assignment must pass. Check returns its problems, or none.
pub trait FallbackActionItemAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &FallbackActionItemAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl FallbackActionItemAssignmentRule for FallbackActionItemAssignmentCodeLengthRule {
    fn check(&self, item: &FallbackActionItemAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemAssignmentRequiredReviewerRule {
    pub message: String,
}

impl FallbackActionItemAssignmentRule for FallbackActionItemAssignmentRequiredReviewerRule {
    fn check(&self, item: &FallbackActionItemAssignment) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemAssignmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackActionItemAssignmentRule for FallbackActionItemAssignmentUpdatedAtRangeRule {
    fn check(&self, item: &FallbackActionItemAssignment) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemAssignmentReviewerLengthRule {
    pub max_length: i64,
}

impl FallbackActionItemAssignmentRule for FallbackActionItemAssignmentReviewerLengthRule {
    fn check(&self, item: &FallbackActionItemAssignment) -> Vec<String> {
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
pub struct FallbackActionItemAssignmentCipherLengthRule {
    pub max_length: i64,
}

impl FallbackActionItemAssignmentRule for FallbackActionItemAssignmentCipherLengthRule {
    fn check(&self, item: &FallbackActionItemAssignment) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback action item assignment.
#[derive(Debug, Clone)]
pub struct FallbackActionItemAssignmentValidator {
    pub rules: Vec<Rc<dyn FallbackActionItemAssignmentRule>>,
}

impl FallbackActionItemAssignmentValidator {
    pub fn validate(&self, item: &FallbackActionItemAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackActionItemAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackActionItemAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_action_item_assignment_validator() -> FallbackActionItemAssignmentValidator {
    FallbackActionItemAssignmentValidator {
        rules: vec![
            Rc::new(FallbackActionItemAssignmentCodeLengthRule { max_length: 64 })
                as Rc<dyn FallbackActionItemAssignmentRule>,
            Rc::new(FallbackActionItemAssignmentRequiredReviewerRule {
                message: "fallback action item assignment needs a value".to_string(),
            }),
            Rc::new(FallbackActionItemAssignmentUpdatedAtRangeRule { min: 80, max: 86400 }),
            Rc::new(FallbackActionItemAssignmentReviewerLengthRule { max_length: 120 }),
            Rc::new(FallbackActionItemAssignmentCipherLengthRule { max_length: 80 }),
        ],
    }
}
