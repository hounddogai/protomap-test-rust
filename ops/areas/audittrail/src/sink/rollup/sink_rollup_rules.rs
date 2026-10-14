use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink rollup must pass. Check returns its problems, or none.
pub trait SinkRollupRule: std::fmt::Debug {
    fn check(&self, item: &SinkRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRollupTitleLengthRule {
    pub max_length: i64,
}

impl SinkRollupRule for SinkRollupTitleLengthRule {
    fn check(&self, item: &SinkRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRollupRequiredTitleRule {
    pub message: String,
}

impl SinkRollupRule for SinkRollupRequiredTitleRule {
    fn check(&self, item: &SinkRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRollupCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkRollupRule for SinkRollupCreatedAtRangeRule {
    fn check(&self, item: &SinkRollup) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRollupCodePrefixRule {
    pub prefix: String,
}

impl SinkRollupRule for SinkRollupCodePrefixRule {
    fn check(&self, item: &SinkRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink rollup.
#[derive(Debug, Clone)]
pub struct SinkRollupValidator {
    pub rules: Vec<Rc<dyn SinkRollupRule>>,
}

impl SinkRollupValidator {
    pub fn validate(&self, item: &SinkRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_rollup_validator() -> SinkRollupValidator {
    SinkRollupValidator {
        rules: vec![
            Rc::new(SinkRollupTitleLengthRule { max_length: 255 }) as Rc<dyn SinkRollupRule>,
            Rc::new(SinkRollupRequiredTitleRule {
                message: "sink rollup needs this field".to_string(),
            }),
            Rc::new(SinkRollupCreatedAtRangeRule { min: 16, max: 80 }),
            Rc::new(SinkRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
