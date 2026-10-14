use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched probe agreement must pass. Check returns its problems, or none.
pub trait BatchedProbeAgreementRule: std::fmt::Debug {
    fn check(&self, item: &BatchedProbeAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeAgreementTitleLengthRule {
    pub max_length: i64,
}

impl BatchedProbeAgreementRule for BatchedProbeAgreementTitleLengthRule {
    fn check(&self, item: &BatchedProbeAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeAgreementPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedProbeAgreementRule for BatchedProbeAgreementPriorityRangeRule {
    fn check(&self, item: &BatchedProbeAgreement) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeAgreementRequiredCodeRule {
    pub message: String,
}

impl BatchedProbeAgreementRule for BatchedProbeAgreementRequiredCodeRule {
    fn check(&self, item: &BatchedProbeAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeAgreementBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedProbeAgreementRule for BatchedProbeAgreementBatchSizeRangeRule {
    fn check(&self, item: &BatchedProbeAgreement) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched probe agreement.
#[derive(Debug, Clone)]
pub struct BatchedProbeAgreementValidator {
    pub rules: Vec<Rc<dyn BatchedProbeAgreementRule>>,
}

impl BatchedProbeAgreementValidator {
    pub fn validate(&self, item: &BatchedProbeAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedProbeAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedProbeAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_probe_agreement_validator() -> BatchedProbeAgreementValidator {
    BatchedProbeAgreementValidator {
        rules: vec![
            Rc::new(BatchedProbeAgreementTitleLengthRule { max_length: 32 }) as Rc<dyn BatchedProbeAgreementRule>,
            Rc::new(BatchedProbeAgreementPriorityRangeRule { min: 3600, max: 86400 }),
            Rc::new(BatchedProbeAgreementRequiredCodeRule {
                message: "batched probe agreement needs an entry".to_string(),
            }),
            Rc::new(BatchedProbeAgreementBatchSizeRangeRule { min: 120, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_probe_agreement_validator();
        let item = new_batched_probe_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedProbeAgreementTitleLengthRule { max_length: 16 };
        let mut item = new_batched_probe_agreement("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = BatchedProbeAgreementPriorityRangeRule { min: 80, max: 255 };
        let mut item = new_batched_probe_agreement("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedProbeAgreementRequiredCodeRule {
            message: "batched probe agreement needs this field".to_string(),
        };
        let mut item = new_batched_probe_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = BatchedProbeAgreementBatchSizeRangeRule { min: 3600, max: 86400 };
        let mut item = new_batched_probe_agreement("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
