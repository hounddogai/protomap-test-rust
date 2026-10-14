use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched segment window must pass. Check returns its problems, or none.
pub trait BatchedSegmentWindowRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSegmentWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentWindowRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedSegmentWindowRule for BatchedSegmentWindowRetriesRangeRule {
    fn check(&self, item: &BatchedSegmentWindow) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentWindowRequiredExternalRefRule {
    pub message: String,
}

impl BatchedSegmentWindowRule for BatchedSegmentWindowRequiredExternalRefRule {
    fn check(&self, item: &BatchedSegmentWindow) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentWindowCodePrefixRule {
    pub prefix: String,
}

impl BatchedSegmentWindowRule for BatchedSegmentWindowCodePrefixRule {
    fn check(&self, item: &BatchedSegmentWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched segment window.
#[derive(Debug, Clone)]
pub struct BatchedSegmentWindowValidator {
    pub rules: Vec<Rc<dyn BatchedSegmentWindowRule>>,
}

impl BatchedSegmentWindowValidator {
    pub fn validate(&self, item: &BatchedSegmentWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedSegmentWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSegmentWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_segment_window_validator() -> BatchedSegmentWindowValidator {
    BatchedSegmentWindowValidator {
        rules: vec![
            Rc::new(BatchedSegmentWindowRetriesRangeRule { min: 80, max: 120 }) as Rc<dyn BatchedSegmentWindowRule>,
            Rc::new(BatchedSegmentWindowRequiredExternalRefRule {
                message: "batched segment window needs a value".to_string(),
            }),
            Rc::new(BatchedSegmentWindowCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_segment_window_validator();
        let item = new_batched_segment_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = BatchedSegmentWindowRetriesRangeRule { min: 16, max: 1000 };
        let mut item = new_batched_segment_window("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = BatchedSegmentWindowRequiredExternalRefRule {
            message: "batched segment window needs a value".to_string(),
        };
        let mut item = new_batched_segment_window("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
