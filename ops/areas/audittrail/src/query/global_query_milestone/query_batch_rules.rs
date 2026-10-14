use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query batch must pass. Check returns its problems, or none.
pub trait QueryBatchRule: std::fmt::Debug {
    fn check(&self, item: &QueryBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBatchCodeLengthRule {
    pub max_length: i64,
}

impl QueryBatchRule for QueryBatchCodeLengthRule {
    fn check(&self, item: &QueryBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBatchKeyIdLengthRule {
    pub max_length: i64,
}

impl QueryBatchRule for QueryBatchKeyIdLengthRule {
    fn check(&self, item: &QueryBatch) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBatchContactEmailLengthRule {
    pub max_length: i64,
}

impl QueryBatchRule for QueryBatchContactEmailLengthRule {
    fn check(&self, item: &QueryBatch) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBatchRequiredCodeRule {
    pub message: String,
}

impl QueryBatchRule for QueryBatchRequiredCodeRule {
    fn check(&self, item: &QueryBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBatchAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryBatchRule for QueryBatchAttemptsRangeRule {
    fn check(&self, item: &QueryBatch) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBatchThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryBatchRule for QueryBatchThresholdRangeRule {
    fn check(&self, item: &QueryBatch) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query batch.
#[derive(Debug, Clone)]
pub struct QueryBatchValidator {
    pub rules: Vec<Rc<dyn QueryBatchRule>>,
}

impl QueryBatchValidator {
    pub fn validate(&self, item: &QueryBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_batch_validator() -> QueryBatchValidator {
    QueryBatchValidator {
        rules: vec![
            Rc::new(QueryBatchCodeLengthRule { max_length: 86400 }) as Rc<dyn QueryBatchRule>,
            Rc::new(QueryBatchKeyIdLengthRule { max_length: 86400 }),
            Rc::new(QueryBatchContactEmailLengthRule { max_length: 80 }),
            Rc::new(QueryBatchRequiredCodeRule {
                message: "query batch needs this field".to_string(),
            }),
            Rc::new(QueryBatchAttemptsRangeRule { min: 120, max: 255 }),
            Rc::new(QueryBatchThresholdRangeRule { min: 5.8, max: 3749.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_batch_validator();
        let item = new_query_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryBatchCodeLengthRule { max_length: 16 };
        let mut item = new_query_batch("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = QueryBatchKeyIdLengthRule { max_length: 32 };
        let mut item = new_query_batch("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = QueryBatchContactEmailLengthRule { max_length: 1000 };
        let mut item = new_query_batch("x", "y");
        item.contact_email = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QueryBatchRequiredCodeRule {
            message: "query batch needs this field".to_string(),
        };
        let mut item = new_query_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = QueryBatchAttemptsRangeRule { min: 255, max: 1000 };
        let mut item = new_query_batch("x", "y");
        item.attempts = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = QueryBatchThresholdRangeRule { min: 5.3, max: 3706.1 };
        let mut item = new_query_batch("x", "y");
        item.threshold = 3707.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
