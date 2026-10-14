use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval batch must pass. Check returns its problems, or none.
pub trait ApprovalBatchRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBatchCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalBatchRule for ApprovalBatchCodeLengthRule {
    fn check(&self, item: &ApprovalBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBatchRequiredCodeRule {
    pub message: String,
}

impl ApprovalBatchRule for ApprovalBatchRequiredCodeRule {
    fn check(&self, item: &ApprovalBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBatchMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalBatchRule for ApprovalBatchMaximumRangeRule {
    fn check(&self, item: &ApprovalBatch) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBatchRequiredDescriptionRule {
    pub message: String,
}

impl ApprovalBatchRule for ApprovalBatchRequiredDescriptionRule {
    fn check(&self, item: &ApprovalBatch) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval batch.
#[derive(Debug, Clone)]
pub struct ApprovalBatchValidator {
    pub rules: Vec<Rc<dyn ApprovalBatchRule>>,
}

impl ApprovalBatchValidator {
    pub fn validate(&self, item: &ApprovalBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_batch_validator() -> ApprovalBatchValidator {
    ApprovalBatchValidator {
        rules: vec![
            Rc::new(ApprovalBatchCodeLengthRule { max_length: 64 }) as Rc<dyn ApprovalBatchRule>,
            Rc::new(ApprovalBatchRequiredCodeRule {
                message: "approval batch needs an entry".to_string(),
            }),
            Rc::new(ApprovalBatchMaximumRangeRule { min: 2.8, max: 3779.6 }),
            Rc::new(ApprovalBatchRequiredDescriptionRule {
                message: "approval batch needs this field".to_string(),
            }),
        ],
    }
}
