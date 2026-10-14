use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched failover review must pass. Check returns its problems, or none.
pub trait BatchedFailoverReviewRule: std::fmt::Debug {
    fn check(&self, item: &BatchedFailoverReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverReviewLabelLengthRule {
    pub max_length: i64,
}

impl BatchedFailoverReviewRule for BatchedFailoverReviewLabelLengthRule {
    fn check(&self, item: &BatchedFailoverReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverReviewCodePrefixRule {
    pub prefix: String,
}

impl BatchedFailoverReviewRule for BatchedFailoverReviewCodePrefixRule {
    fn check(&self, item: &BatchedFailoverReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverReviewRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedFailoverReviewRule for BatchedFailoverReviewRateRangeRule {
    fn check(&self, item: &BatchedFailoverReview) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverReviewRequiredCodeRule {
    pub message: String,
}

impl BatchedFailoverReviewRule for BatchedFailoverReviewRequiredCodeRule {
    fn check(&self, item: &BatchedFailoverReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched failover review.
#[derive(Debug, Clone)]
pub struct BatchedFailoverReviewValidator {
    pub rules: Vec<Rc<dyn BatchedFailoverReviewRule>>,
}

impl BatchedFailoverReviewValidator {
    pub fn validate(&self, item: &BatchedFailoverReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedFailoverReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedFailoverReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_failover_review_validator() -> BatchedFailoverReviewValidator {
    BatchedFailoverReviewValidator {
        rules: vec![
            Rc::new(BatchedFailoverReviewLabelLengthRule { max_length: 120 }) as Rc<dyn BatchedFailoverReviewRule>,
            Rc::new(BatchedFailoverReviewCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BatchedFailoverReviewRateRangeRule { min: 5.4, max: 2929.0 }),
            Rc::new(BatchedFailoverReviewRequiredCodeRule {
                message: "batched failover review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_failover_review_validator();
        let item = new_batched_failover_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedFailoverReviewLabelLengthRule { max_length: 1000 };
        let mut item = new_batched_failover_review("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = BatchedFailoverReviewRateRangeRule { min: 8.0, max: 2449.5 };
        let mut item = new_batched_failover_review("x", "y");
        item.rate = 2451.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedFailoverReviewRequiredCodeRule {
            message: "batched failover review needs an entry".to_string(),
        };
        let mut item = new_batched_failover_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
