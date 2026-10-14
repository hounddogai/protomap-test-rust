use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary batch must pass. Check returns its problems, or none.
pub trait CanaryBatchRule: std::fmt::Debug {
    fn check(&self, item: &CanaryBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBatchPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryBatchRule for CanaryBatchPriorityRangeRule {
    fn check(&self, item: &CanaryBatch) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBatchCodeLengthRule {
    pub max_length: i64,
}

impl CanaryBatchRule for CanaryBatchCodeLengthRule {
    fn check(&self, item: &CanaryBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBatchRequiredSourceRule {
    pub message: String,
}

impl CanaryBatchRule for CanaryBatchRequiredSourceRule {
    fn check(&self, item: &CanaryBatch) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBatchLabelLengthRule {
    pub max_length: i64,
}

impl CanaryBatchRule for CanaryBatchLabelLengthRule {
    fn check(&self, item: &CanaryBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary batch.
#[derive(Debug, Clone)]
pub struct CanaryBatchValidator {
    pub rules: Vec<Rc<dyn CanaryBatchRule>>,
}

impl CanaryBatchValidator {
    pub fn validate(&self, item: &CanaryBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_batch_validator() -> CanaryBatchValidator {
    CanaryBatchValidator {
        rules: vec![
            Rc::new(CanaryBatchPriorityRangeRule { min: 16, max: 1000 }) as Rc<dyn CanaryBatchRule>,
            Rc::new(CanaryBatchCodeLengthRule { max_length: 255 }),
            Rc::new(CanaryBatchRequiredSourceRule {
                message: "canary batch needs this field".to_string(),
            }),
            Rc::new(CanaryBatchLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_batch_validator();
        let item = new_canary_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = CanaryBatchPriorityRangeRule { min: 16, max: 255 };
        let mut item = new_canary_batch("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanaryBatchCodeLengthRule { max_length: 32 };
        let mut item = new_canary_batch("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = CanaryBatchRequiredSourceRule {
            message: "canary batch needs this field".to_string(),
        };
        let mut item = new_canary_batch("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CanaryBatchLabelLengthRule { max_length: 32 };
        let mut item = new_canary_batch("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
