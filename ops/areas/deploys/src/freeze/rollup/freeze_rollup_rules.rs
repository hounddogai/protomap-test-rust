use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze rollup must pass. Check returns its problems, or none.
pub trait FreezeRollupRule: std::fmt::Debug {
    fn check(&self, item: &FreezeRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRollupTitleLengthRule {
    pub max_length: i64,
}

impl FreezeRollupRule for FreezeRollupTitleLengthRule {
    fn check(&self, item: &FreezeRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRollupRequiredTitleRule {
    pub message: String,
}

impl FreezeRollupRule for FreezeRollupRequiredTitleRule {
    fn check(&self, item: &FreezeRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRollupArchivedGuardRule {
    pub limit: f64,
}

impl FreezeRollupRule for FreezeRollupArchivedGuardRule {
    fn check(&self, item: &FreezeRollup) -> Vec<String> {
        if item.archived && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while archived", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRollupAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeRollupRule for FreezeRollupAmountRangeRule {
    fn check(&self, item: &FreezeRollup) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRollupRequiredCodeRule {
    pub message: String,
}

impl FreezeRollupRule for FreezeRollupRequiredCodeRule {
    fn check(&self, item: &FreezeRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze rollup.
#[derive(Debug, Clone)]
pub struct FreezeRollupValidator {
    pub rules: Vec<Rc<dyn FreezeRollupRule>>,
}

impl FreezeRollupValidator {
    pub fn validate(&self, item: &FreezeRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_rollup_validator() -> FreezeRollupValidator {
    FreezeRollupValidator {
        rules: vec![
            Rc::new(FreezeRollupTitleLengthRule { max_length: 64 }) as Rc<dyn FreezeRollupRule>,
            Rc::new(FreezeRollupRequiredTitleRule {
                message: "freeze rollup needs an entry".to_string(),
            }),
            Rc::new(FreezeRollupArchivedGuardRule { limit: 5.2 }),
            Rc::new(FreezeRollupAmountRangeRule { min: 3.3, max: 4789.6 }),
            Rc::new(FreezeRollupRequiredCodeRule {
                message: "freeze rollup needs an entry".to_string(),
            }),
        ],
    }
}
