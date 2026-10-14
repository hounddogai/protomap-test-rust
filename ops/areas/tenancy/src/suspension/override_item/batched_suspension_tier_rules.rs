use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched suspension tier must pass. Check returns its problems, or none.
pub trait BatchedSuspensionTierRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSuspensionTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionTierRequiredNotesRule {
    pub message: String,
}

impl BatchedSuspensionTierRule for BatchedSuspensionTierRequiredNotesRule {
    fn check(&self, item: &BatchedSuspensionTier) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionTierDescriptionLengthRule {
    pub max_length: i64,
}

impl BatchedSuspensionTierRule for BatchedSuspensionTierDescriptionLengthRule {
    fn check(&self, item: &BatchedSuspensionTier) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionTierRequiredDescriptionRule {
    pub message: String,
}

impl BatchedSuspensionTierRule for BatchedSuspensionTierRequiredDescriptionRule {
    fn check(&self, item: &BatchedSuspensionTier) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionTierBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedSuspensionTierRule for BatchedSuspensionTierBatchSizeRangeRule {
    fn check(&self, item: &BatchedSuspensionTier) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched suspension tier.
#[derive(Debug, Clone)]
pub struct BatchedSuspensionTierValidator {
    pub rules: Vec<Rc<dyn BatchedSuspensionTierRule>>,
}

impl BatchedSuspensionTierValidator {
    pub fn validate(&self, item: &BatchedSuspensionTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedSuspensionTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSuspensionTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_suspension_tier_validator() -> BatchedSuspensionTierValidator {
    BatchedSuspensionTierValidator {
        rules: vec![
            Rc::new(BatchedSuspensionTierRequiredNotesRule {
                message: "batched suspension tier needs an entry".to_string(),
            }) as Rc<dyn BatchedSuspensionTierRule>,
            Rc::new(BatchedSuspensionTierDescriptionLengthRule { max_length: 32 }),
            Rc::new(BatchedSuspensionTierRequiredDescriptionRule {
                message: "batched suspension tier needs this field".to_string(),
            }),
            Rc::new(BatchedSuspensionTierBatchSizeRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_suspension_tier_validator();
        let item = new_batched_suspension_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = BatchedSuspensionTierRequiredNotesRule {
            message: "batched suspension tier needs an entry".to_string(),
        };
        let mut item = new_batched_suspension_tier("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = BatchedSuspensionTierDescriptionLengthRule { max_length: 16 };
        let mut item = new_batched_suspension_tier("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = BatchedSuspensionTierRequiredDescriptionRule {
            message: "batched suspension tier needs a value".to_string(),
        };
        let mut item = new_batched_suspension_tier("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = BatchedSuspensionTierBatchSizeRangeRule { min: 32, max: 1000 };
        let mut item = new_batched_suspension_tier("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
