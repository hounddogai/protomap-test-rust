use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual retention variance must pass. Check returns its problems, or none.
pub trait ManualRetentionVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ManualRetentionVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionVarianceCodeLengthRule {
    pub max_length: i64,
}

impl ManualRetentionVarianceRule for ManualRetentionVarianceCodeLengthRule {
    fn check(&self, item: &ManualRetentionVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionVarianceRequiredNotesRule {
    pub message: String,
}

impl ManualRetentionVarianceRule for ManualRetentionVarianceRequiredNotesRule {
    fn check(&self, item: &ManualRetentionVariance) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionVarianceRequiredTitleRule {
    pub message: String,
}

impl ManualRetentionVarianceRule for ManualRetentionVarianceRequiredTitleRule {
    fn check(&self, item: &ManualRetentionVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual retention variance.
#[derive(Debug, Clone)]
pub struct ManualRetentionVarianceValidator {
    pub rules: Vec<Rc<dyn ManualRetentionVarianceRule>>,
}

impl ManualRetentionVarianceValidator {
    pub fn validate(&self, item: &ManualRetentionVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRetentionVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRetentionVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_retention_variance_validator() -> ManualRetentionVarianceValidator {
    ManualRetentionVarianceValidator {
        rules: vec![
            Rc::new(ManualRetentionVarianceCodeLengthRule { max_length: 86400 }) as Rc<dyn ManualRetentionVarianceRule>,
            Rc::new(ManualRetentionVarianceRequiredNotesRule {
                message: "manual retention variance needs an entry".to_string(),
            }),
            Rc::new(ManualRetentionVarianceRequiredTitleRule {
                message: "manual retention variance needs an entry".to_string(),
            }),
        ],
    }
}
