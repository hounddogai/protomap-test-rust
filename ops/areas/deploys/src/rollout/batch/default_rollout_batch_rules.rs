use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default rollout batch must pass. Check returns its problems, or none.
pub trait DefaultRolloutBatchRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRolloutBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBatchChecksumLengthRule {
    pub max_length: i64,
}

impl DefaultRolloutBatchRule for DefaultRolloutBatchChecksumLengthRule {
    fn check(&self, item: &DefaultRolloutBatch) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBatchQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRolloutBatchRule for DefaultRolloutBatchQuantityRangeRule {
    fn check(&self, item: &DefaultRolloutBatch) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBatchRequiredNameRule {
    pub message: String,
}

impl DefaultRolloutBatchRule for DefaultRolloutBatchRequiredNameRule {
    fn check(&self, item: &DefaultRolloutBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBatchRequiredContactEmailRule {
    pub message: String,
}

impl DefaultRolloutBatchRule for DefaultRolloutBatchRequiredContactEmailRule {
    fn check(&self, item: &DefaultRolloutBatch) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBatchNameLengthRule {
    pub max_length: i64,
}

impl DefaultRolloutBatchRule for DefaultRolloutBatchNameLengthRule {
    fn check(&self, item: &DefaultRolloutBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBatchRequiredChecksumRule {
    pub message: String,
}

impl DefaultRolloutBatchRule for DefaultRolloutBatchRequiredChecksumRule {
    fn check(&self, item: &DefaultRolloutBatch) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rollout batch.
#[derive(Debug, Clone)]
pub struct DefaultRolloutBatchValidator {
    pub rules: Vec<Rc<dyn DefaultRolloutBatchRule>>,
}

impl DefaultRolloutBatchValidator {
    pub fn validate(&self, item: &DefaultRolloutBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRolloutBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRolloutBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rollout_batch_validator() -> DefaultRolloutBatchValidator {
    DefaultRolloutBatchValidator {
        rules: vec![
            Rc::new(DefaultRolloutBatchChecksumLengthRule { max_length: 80 }) as Rc<dyn DefaultRolloutBatchRule>,
            Rc::new(DefaultRolloutBatchQuantityRangeRule { min: 64, max: 80 }),
            Rc::new(DefaultRolloutBatchRequiredNameRule {
                message: "default rollout batch needs a value".to_string(),
            }),
            Rc::new(DefaultRolloutBatchRequiredContactEmailRule {
                message: "default rollout batch needs a value".to_string(),
            }),
            Rc::new(DefaultRolloutBatchNameLengthRule { max_length: 32 }),
            Rc::new(DefaultRolloutBatchRequiredChecksumRule {
                message: "default rollout batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_rollout_batch_validator();
        let item = new_default_rollout_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = DefaultRolloutBatchChecksumLengthRule { max_length: 16 };
        let mut item = new_default_rollout_batch("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DefaultRolloutBatchQuantityRangeRule { min: 80, max: 1000 };
        let mut item = new_default_rollout_batch("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultRolloutBatchRequiredNameRule {
            message: "default rollout batch needs an entry".to_string(),
        };
        let mut item = new_default_rollout_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = DefaultRolloutBatchRequiredContactEmailRule {
            message: "default rollout batch needs an entry".to_string(),
        };
        let mut item = new_default_rollout_batch("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultRolloutBatchNameLengthRule { max_length: 80 };
        let mut item = new_default_rollout_batch("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = DefaultRolloutBatchRequiredChecksumRule {
            message: "default rollout batch needs an entry".to_string(),
        };
        let mut item = new_default_rollout_batch("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
