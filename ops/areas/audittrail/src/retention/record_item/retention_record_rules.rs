use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention record must pass. Check returns its problems, or none.
pub trait RetentionRecordRule: std::fmt::Debug {
    fn check(&self, item: &RetentionRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRecordTitleLengthRule {
    pub max_length: i64,
}

impl RetentionRecordRule for RetentionRecordTitleLengthRule {
    fn check(&self, item: &RetentionRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRecordAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionRecordRule for RetentionRecordAttemptsRangeRule {
    fn check(&self, item: &RetentionRecord) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRecordRequiredCodeRule {
    pub message: String,
}

impl RetentionRecordRule for RetentionRecordRequiredCodeRule {
    fn check(&self, item: &RetentionRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRecordRequiredClusterRule {
    pub message: String,
}

impl RetentionRecordRule for RetentionRecordRequiredClusterRule {
    fn check(&self, item: &RetentionRecord) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention record.
#[derive(Debug, Clone)]
pub struct RetentionRecordValidator {
    pub rules: Vec<Rc<dyn RetentionRecordRule>>,
}

impl RetentionRecordValidator {
    pub fn validate(&self, item: &RetentionRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_record_validator() -> RetentionRecordValidator {
    RetentionRecordValidator {
        rules: vec![
            Rc::new(RetentionRecordTitleLengthRule { max_length: 80 }) as Rc<dyn RetentionRecordRule>,
            Rc::new(RetentionRecordAttemptsRangeRule { min: 80, max: 86400 }),
            Rc::new(RetentionRecordRequiredCodeRule {
                message: "retention record needs this field".to_string(),
            }),
            Rc::new(RetentionRecordRequiredClusterRule {
                message: "retention record needs a value".to_string(),
            }),
        ],
    }
}
