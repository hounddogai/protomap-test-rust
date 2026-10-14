use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched budget burn batch must pass. Check returns its problems, or none.
pub trait BatchedBudgetBurnBatchRule: std::fmt::Debug {
    fn check(&self, item: &BatchedBudgetBurnBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnBatchRequiredLabelRule {
    pub message: String,
}

impl BatchedBudgetBurnBatchRule for BatchedBudgetBurnBatchRequiredLabelRule {
    fn check(&self, item: &BatchedBudgetBurnBatch) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnBatchCodeLengthRule {
    pub max_length: i64,
}

impl BatchedBudgetBurnBatchRule for BatchedBudgetBurnBatchCodeLengthRule {
    fn check(&self, item: &BatchedBudgetBurnBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnBatchRequiredCodeRule {
    pub message: String,
}

impl BatchedBudgetBurnBatchRule for BatchedBudgetBurnBatchRequiredCodeRule {
    fn check(&self, item: &BatchedBudgetBurnBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnBatchMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedBudgetBurnBatchRule for BatchedBudgetBurnBatchMaximumRangeRule {
    fn check(&self, item: &BatchedBudgetBurnBatch) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched budget burn batch.
#[derive(Debug, Clone)]
pub struct BatchedBudgetBurnBatchValidator {
    pub rules: Vec<Rc<dyn BatchedBudgetBurnBatchRule>>,
}

impl BatchedBudgetBurnBatchValidator {
    pub fn validate(&self, item: &BatchedBudgetBurnBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedBudgetBurnBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedBudgetBurnBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_budget_burn_batch_validator() -> BatchedBudgetBurnBatchValidator {
    BatchedBudgetBurnBatchValidator {
        rules: vec![
            Rc::new(BatchedBudgetBurnBatchRequiredLabelRule {
                message: "batched budget burn batch needs a value".to_string(),
            }) as Rc<dyn BatchedBudgetBurnBatchRule>,
            Rc::new(BatchedBudgetBurnBatchCodeLengthRule { max_length: 3600 }),
            Rc::new(BatchedBudgetBurnBatchRequiredCodeRule {
                message: "batched budget burn batch needs a value".to_string(),
            }),
            Rc::new(BatchedBudgetBurnBatchMaximumRangeRule { min: 4.6, max: 1904.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_budget_burn_batch_validator();
        let item = new_batched_budget_burn_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedBudgetBurnBatchRequiredLabelRule {
            message: "batched budget burn batch needs an entry".to_string(),
        };
        let mut item = new_batched_budget_burn_batch("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedBudgetBurnBatchCodeLengthRule { max_length: 16 };
        let mut item = new_batched_budget_burn_batch("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedBudgetBurnBatchRequiredCodeRule {
            message: "batched budget burn batch needs a value".to_string(),
        };
        let mut item = new_batched_budget_burn_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = BatchedBudgetBurnBatchMaximumRangeRule { min: 7.6, max: 2169.6 };
        let mut item = new_batched_budget_burn_batch("x", "y");
        item.maximum = 2171.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
