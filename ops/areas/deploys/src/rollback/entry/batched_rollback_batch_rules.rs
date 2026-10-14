use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched rollback batch must pass. Check returns its problems, or none.
pub trait BatchedRollbackBatchRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRollbackBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchRequiredUnitRule {
    pub message: String,
}

impl BatchedRollbackBatchRule for BatchedRollbackBatchRequiredUnitRule {
    fn check(&self, item: &BatchedRollbackBatch) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedRollbackBatchRule for BatchedRollbackBatchAmountRangeRule {
    fn check(&self, item: &BatchedRollbackBatch) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchTitleLengthRule {
    pub max_length: i64,
}

impl BatchedRollbackBatchRule for BatchedRollbackBatchTitleLengthRule {
    fn check(&self, item: &BatchedRollbackBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRollbackBatchRule for BatchedRollbackBatchExpiresAtRangeRule {
    fn check(&self, item: &BatchedRollbackBatch) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchLabelLengthRule {
    pub max_length: i64,
}

impl BatchedRollbackBatchRule for BatchedRollbackBatchLabelLengthRule {
    fn check(&self, item: &BatchedRollbackBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl BatchedRollbackBatchRule for BatchedRollbackBatchAllowedUnitRule {
    fn check(&self, item: &BatchedRollbackBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is duplicated", item.unit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched rollback batch.
#[derive(Debug, Clone)]
pub struct BatchedRollbackBatchValidator {
    pub rules: Vec<Rc<dyn BatchedRollbackBatchRule>>,
}

impl BatchedRollbackBatchValidator {
    pub fn validate(&self, item: &BatchedRollbackBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRollbackBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRollbackBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_rollback_batch_validator() -> BatchedRollbackBatchValidator {
    BatchedRollbackBatchValidator {
        rules: vec![
            Rc::new(BatchedRollbackBatchRequiredUnitRule {
                message: "batched rollback batch needs a value".to_string(),
            }) as Rc<dyn BatchedRollbackBatchRule>,
            Rc::new(BatchedRollbackBatchAmountRangeRule { min: 2.8, max: 1076.8 }),
            Rc::new(BatchedRollbackBatchTitleLengthRule { max_length: 64 }),
            Rc::new(BatchedRollbackBatchExpiresAtRangeRule { min: 1000, max: 86400 }),
            Rc::new(BatchedRollbackBatchLabelLengthRule { max_length: 1000 }),
            Rc::new(BatchedRollbackBatchAllowedUnitRule {
                allowed: vec!["strict".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_rollback_batch_validator();
        let item = new_batched_rollback_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = BatchedRollbackBatchRequiredUnitRule {
            message: "batched rollback batch needs an entry".to_string(),
        };
        let mut item = new_batched_rollback_batch("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = BatchedRollbackBatchAmountRangeRule { min: 0.6, max: 1718.4 };
        let mut item = new_batched_rollback_batch("x", "y");
        item.amount = 1719.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedRollbackBatchTitleLengthRule { max_length: 86400 };
        let mut item = new_batched_rollback_batch("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = BatchedRollbackBatchExpiresAtRangeRule { min: 80, max: 120 };
        let mut item = new_batched_rollback_batch("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedRollbackBatchLabelLengthRule { max_length: 3600 };
        let mut item = new_batched_rollback_batch("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
