use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual allowlist batch must pass. Check returns its problems, or none.
pub trait ManualAllowlistBatchRule: std::fmt::Debug {
    fn check(&self, item: &ManualAllowlistBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBatchAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualAllowlistBatchRule for ManualAllowlistBatchAmountRangeRule {
    fn check(&self, item: &ManualAllowlistBatch) -> Vec<String> {
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
pub struct ManualAllowlistBatchExternalRefLengthRule {
    pub max_length: i64,
}

impl ManualAllowlistBatchRule for ManualAllowlistBatchExternalRefLengthRule {
    fn check(&self, item: &ManualAllowlistBatch) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBatchTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualAllowlistBatchRule for ManualAllowlistBatchTtlSecondsRangeRule {
    fn check(&self, item: &ManualAllowlistBatch) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual allowlist batch.
#[derive(Debug, Clone)]
pub struct ManualAllowlistBatchValidator {
    pub rules: Vec<Rc<dyn ManualAllowlistBatchRule>>,
}

impl ManualAllowlistBatchValidator {
    pub fn validate(&self, item: &ManualAllowlistBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualAllowlistBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAllowlistBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_allowlist_batch_validator() -> ManualAllowlistBatchValidator {
    ManualAllowlistBatchValidator {
        rules: vec![
            Rc::new(ManualAllowlistBatchAmountRangeRule { min: 2.8, max: 1872.9 }) as Rc<dyn ManualAllowlistBatchRule>,
            Rc::new(ManualAllowlistBatchExternalRefLengthRule { max_length: 1000 }),
            Rc::new(ManualAllowlistBatchTtlSecondsRangeRule { min: 32, max: 80 }),
        ],
    }
}
