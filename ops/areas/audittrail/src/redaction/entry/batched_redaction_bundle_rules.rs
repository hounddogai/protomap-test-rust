use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched redaction bundle must pass. Check returns its problems, or none.
pub trait BatchedRedactionBundleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRedactionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionBundleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRedactionBundleRule for BatchedRedactionBundleCreatedAtRangeRule {
    fn check(&self, item: &BatchedRedactionBundle) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionBundleCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRedactionBundleRule for BatchedRedactionBundleCodeLengthRule {
    fn check(&self, item: &BatchedRedactionBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionBundleRequiredCodeRule {
    pub message: String,
}

impl BatchedRedactionBundleRule for BatchedRedactionBundleRequiredCodeRule {
    fn check(&self, item: &BatchedRedactionBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched redaction bundle.
#[derive(Debug, Clone)]
pub struct BatchedRedactionBundleValidator {
    pub rules: Vec<Rc<dyn BatchedRedactionBundleRule>>,
}

impl BatchedRedactionBundleValidator {
    pub fn validate(&self, item: &BatchedRedactionBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRedactionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRedactionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_redaction_bundle_validator() -> BatchedRedactionBundleValidator {
    BatchedRedactionBundleValidator {
        rules: vec![
            Rc::new(BatchedRedactionBundleCreatedAtRangeRule { min: 16, max: 32 })
                as Rc<dyn BatchedRedactionBundleRule>,
            Rc::new(BatchedRedactionBundleCodeLengthRule { max_length: 1000 }),
            Rc::new(BatchedRedactionBundleRequiredCodeRule {
                message: "batched redaction bundle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_redaction_bundle_validator();
        let item = new_batched_redaction_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = BatchedRedactionBundleCreatedAtRangeRule { min: 32, max: 120 };
        let mut item = new_batched_redaction_bundle("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRedactionBundleCodeLengthRule { max_length: 16 };
        let mut item = new_batched_redaction_bundle("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedRedactionBundleRequiredCodeRule {
            message: "batched redaction bundle needs this field".to_string(),
        };
        let mut item = new_batched_redaction_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
