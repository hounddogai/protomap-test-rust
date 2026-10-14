use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted rollback batch must pass. Check returns its problems, or none.
pub trait WeightedRollbackBatchRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRollbackBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBatchPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedRollbackBatchRule for WeightedRollbackBatchPriorityRangeRule {
    fn check(&self, item: &WeightedRollbackBatch) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBatchRequiredCodeRule {
    pub message: String,
}

impl WeightedRollbackBatchRule for WeightedRollbackBatchRequiredCodeRule {
    fn check(&self, item: &WeightedRollbackBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBatchFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRollbackBatchRule for WeightedRollbackBatchFloorValueRangeRule {
    fn check(&self, item: &WeightedRollbackBatch) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBatchAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl WeightedRollbackBatchRule for WeightedRollbackBatchAllowedUnitRule {
    fn check(&self, item: &WeightedRollbackBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is expired", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBatchFingerprintLengthRule {
    pub max_length: i64,
}

impl WeightedRollbackBatchRule for WeightedRollbackBatchFingerprintLengthRule {
    fn check(&self, item: &WeightedRollbackBatch) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted rollback batch.
#[derive(Debug, Clone)]
pub struct WeightedRollbackBatchValidator {
    pub rules: Vec<Rc<dyn WeightedRollbackBatchRule>>,
}

impl WeightedRollbackBatchValidator {
    pub fn validate(&self, item: &WeightedRollbackBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRollbackBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRollbackBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rollback_batch_validator() -> WeightedRollbackBatchValidator {
    WeightedRollbackBatchValidator {
        rules: vec![
            Rc::new(WeightedRollbackBatchPriorityRangeRule { min: 32, max: 64 }) as Rc<dyn WeightedRollbackBatchRule>,
            Rc::new(WeightedRollbackBatchRequiredCodeRule {
                message: "weighted rollback batch needs an entry".to_string(),
            }),
            Rc::new(WeightedRollbackBatchFloorValueRangeRule { min: 9.3, max: 1207.5 }),
            Rc::new(WeightedRollbackBatchAllowedUnitRule { allowed: Vec::new() }),
            Rc::new(WeightedRollbackBatchFingerprintLengthRule { max_length: 64 }),
        ],
    }
}
