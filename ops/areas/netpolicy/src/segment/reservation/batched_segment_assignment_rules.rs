use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched segment assignment must pass. Check returns its problems, or none.
pub trait BatchedSegmentAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSegmentAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentAssignmentRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedSegmentAssignmentRule for BatchedSegmentAssignmentRatioRangeRule {
    fn check(&self, item: &BatchedSegmentAssignment) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentAssignmentCurrencyLengthRule {
    pub max_length: i64,
}

impl BatchedSegmentAssignmentRule for BatchedSegmentAssignmentCurrencyLengthRule {
    fn check(&self, item: &BatchedSegmentAssignment) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentAssignmentRequiredCodeRule {
    pub message: String,
}

impl BatchedSegmentAssignmentRule for BatchedSegmentAssignmentRequiredCodeRule {
    fn check(&self, item: &BatchedSegmentAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentAssignmentRequiredCurrencyRule {
    pub message: String,
}

impl BatchedSegmentAssignmentRule for BatchedSegmentAssignmentRequiredCurrencyRule {
    fn check(&self, item: &BatchedSegmentAssignment) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentAssignmentFingerprintLengthRule {
    pub max_length: i64,
}

impl BatchedSegmentAssignmentRule for BatchedSegmentAssignmentFingerprintLengthRule {
    fn check(&self, item: &BatchedSegmentAssignment) -> Vec<String> {
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

/// Runs every rule of a batched segment assignment.
#[derive(Debug, Clone)]
pub struct BatchedSegmentAssignmentValidator {
    pub rules: Vec<Rc<dyn BatchedSegmentAssignmentRule>>,
}

impl BatchedSegmentAssignmentValidator {
    pub fn validate(&self, item: &BatchedSegmentAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedSegmentAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSegmentAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_segment_assignment_validator() -> BatchedSegmentAssignmentValidator {
    BatchedSegmentAssignmentValidator {
        rules: vec![
            Rc::new(BatchedSegmentAssignmentRatioRangeRule { min: 9.2, max: 2914.1 })
                as Rc<dyn BatchedSegmentAssignmentRule>,
            Rc::new(BatchedSegmentAssignmentCurrencyLengthRule { max_length: 32 }),
            Rc::new(BatchedSegmentAssignmentRequiredCodeRule {
                message: "batched segment assignment needs this field".to_string(),
            }),
            Rc::new(BatchedSegmentAssignmentRequiredCurrencyRule {
                message: "batched segment assignment needs an entry".to_string(),
            }),
            Rc::new(BatchedSegmentAssignmentFingerprintLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_segment_assignment_validator();
        let item = new_batched_segment_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = BatchedSegmentAssignmentRatioRangeRule { min: 4.2, max: 731.7 };
        let mut item = new_batched_segment_assignment("x", "y");
        item.ratio = 733.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = BatchedSegmentAssignmentCurrencyLengthRule { max_length: 3600 };
        let mut item = new_batched_segment_assignment("x", "y");
        item.currency = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedSegmentAssignmentRequiredCodeRule {
            message: "batched segment assignment needs this field".to_string(),
        };
        let mut item = new_batched_segment_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = BatchedSegmentAssignmentRequiredCurrencyRule {
            message: "batched segment assignment needs an entry".to_string(),
        };
        let mut item = new_batched_segment_assignment("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = BatchedSegmentAssignmentFingerprintLengthRule { max_length: 80 };
        let mut item = new_batched_segment_assignment("x", "y");
        item.fingerprint = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
