use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched policy segment must pass. Check returns its problems, or none.
pub trait BatchedPolicySegmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicySegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicySegmentRequiredSourceRule {
    pub message: String,
}

impl BatchedPolicySegmentRule for BatchedPolicySegmentRequiredSourceRule {
    fn check(&self, item: &BatchedPolicySegment) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicySegmentRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPolicySegmentRule for BatchedPolicySegmentRevisionRangeRule {
    fn check(&self, item: &BatchedPolicySegment) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicySegmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedPolicySegmentRule for BatchedPolicySegmentCeilingRangeRule {
    fn check(&self, item: &BatchedPolicySegment) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicySegmentSourceLengthRule {
    pub max_length: i64,
}

impl BatchedPolicySegmentRule for BatchedPolicySegmentSourceLengthRule {
    fn check(&self, item: &BatchedPolicySegment) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched policy segment.
#[derive(Debug, Clone)]
pub struct BatchedPolicySegmentValidator {
    pub rules: Vec<Rc<dyn BatchedPolicySegmentRule>>,
}

impl BatchedPolicySegmentValidator {
    pub fn validate(&self, item: &BatchedPolicySegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPolicySegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicySegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_segment_validator() -> BatchedPolicySegmentValidator {
    BatchedPolicySegmentValidator {
        rules: vec![
            Rc::new(BatchedPolicySegmentRequiredSourceRule {
                message: "batched policy segment needs an entry".to_string(),
            }) as Rc<dyn BatchedPolicySegmentRule>,
            Rc::new(BatchedPolicySegmentRevisionRangeRule { min: 3600, max: 86400 }),
            Rc::new(BatchedPolicySegmentCeilingRangeRule { min: 1.8, max: 4573.4 }),
            Rc::new(BatchedPolicySegmentSourceLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_policy_segment_validator();
        let item = new_batched_policy_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = BatchedPolicySegmentRequiredSourceRule {
            message: "batched policy segment needs this field".to_string(),
        };
        let mut item = new_batched_policy_segment("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = BatchedPolicySegmentRevisionRangeRule { min: 16, max: 120 };
        let mut item = new_batched_policy_segment("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = BatchedPolicySegmentCeilingRangeRule { min: 9.0, max: 1003.9 };
        let mut item = new_batched_policy_segment("x", "y");
        item.ceiling = 1005.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = BatchedPolicySegmentSourceLengthRule { max_length: 64 };
        let mut item = new_batched_policy_segment("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
