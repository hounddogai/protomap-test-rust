use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending retention rollup must pass. Check returns its problems, or none.
pub trait PendingRetentionRollupRule: std::fmt::Debug {
    fn check(&self, item: &PendingRetentionRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionRollupRequiredNameRule {
    pub message: String,
}

impl PendingRetentionRollupRule for PendingRetentionRollupRequiredNameRule {
    fn check(&self, item: &PendingRetentionRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionRollupNameLengthRule {
    pub max_length: i64,
}

impl PendingRetentionRollupRule for PendingRetentionRollupNameLengthRule {
    fn check(&self, item: &PendingRetentionRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionRollupRequiredDescriptionRule {
    pub message: String,
}

impl PendingRetentionRollupRule for PendingRetentionRollupRequiredDescriptionRule {
    fn check(&self, item: &PendingRetentionRollup) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionRollupWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRetentionRollupRule for PendingRetentionRollupWeightRangeRule {
    fn check(&self, item: &PendingRetentionRollup) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending retention rollup.
#[derive(Debug, Clone)]
pub struct PendingRetentionRollupValidator {
    pub rules: Vec<Rc<dyn PendingRetentionRollupRule>>,
}

impl PendingRetentionRollupValidator {
    pub fn validate(&self, item: &PendingRetentionRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingRetentionRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRetentionRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_retention_rollup_validator() -> PendingRetentionRollupValidator {
    PendingRetentionRollupValidator {
        rules: vec![
            Rc::new(PendingRetentionRollupRequiredNameRule {
                message: "pending retention rollup needs an entry".to_string(),
            }) as Rc<dyn PendingRetentionRollupRule>,
            Rc::new(PendingRetentionRollupNameLengthRule { max_length: 86400 }),
            Rc::new(PendingRetentionRollupRequiredDescriptionRule {
                message: "pending retention rollup needs a value".to_string(),
            }),
            Rc::new(PendingRetentionRollupWeightRangeRule { min: 9.6, max: 4183.6 }),
        ],
    }
}
