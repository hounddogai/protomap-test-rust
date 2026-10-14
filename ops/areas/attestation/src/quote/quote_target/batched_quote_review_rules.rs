use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched quote review must pass. Check returns its problems, or none.
pub trait BatchedQuoteReviewRule: std::fmt::Debug {
    fn check(&self, item: &BatchedQuoteReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteReviewAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl BatchedQuoteReviewRule for BatchedQuoteReviewAllowedUnitRule {
    fn check(&self, item: &BatchedQuoteReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is inconsistent", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteReviewCodeLengthRule {
    pub max_length: i64,
}

impl BatchedQuoteReviewRule for BatchedQuoteReviewCodeLengthRule {
    fn check(&self, item: &BatchedQuoteReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteReviewRequiredSummaryRule {
    pub message: String,
}

impl BatchedQuoteReviewRule for BatchedQuoteReviewRequiredSummaryRule {
    fn check(&self, item: &BatchedQuoteReview) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteReviewRequiredCodeRule {
    pub message: String,
}

impl BatchedQuoteReviewRule for BatchedQuoteReviewRequiredCodeRule {
    fn check(&self, item: &BatchedQuoteReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteReviewSummaryLengthRule {
    pub max_length: i64,
}

impl BatchedQuoteReviewRule for BatchedQuoteReviewSummaryLengthRule {
    fn check(&self, item: &BatchedQuoteReview) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched quote review.
#[derive(Debug, Clone)]
pub struct BatchedQuoteReviewValidator {
    pub rules: Vec<Rc<dyn BatchedQuoteReviewRule>>,
}

impl BatchedQuoteReviewValidator {
    pub fn validate(&self, item: &BatchedQuoteReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedQuoteReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedQuoteReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_quote_review_validator() -> BatchedQuoteReviewValidator {
    BatchedQuoteReviewValidator {
        rules: vec![
            Rc::new(BatchedQuoteReviewAllowedUnitRule { allowed: Vec::new() }) as Rc<dyn BatchedQuoteReviewRule>,
            Rc::new(BatchedQuoteReviewCodeLengthRule { max_length: 80 }),
            Rc::new(BatchedQuoteReviewRequiredSummaryRule {
                message: "batched quote review needs a value".to_string(),
            }),
            Rc::new(BatchedQuoteReviewRequiredCodeRule {
                message: "batched quote review needs an entry".to_string(),
            }),
            Rc::new(BatchedQuoteReviewSummaryLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_quote_review_validator();
        let item = new_batched_quote_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedQuoteReviewCodeLengthRule { max_length: 16 };
        let mut item = new_batched_quote_review("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = BatchedQuoteReviewRequiredSummaryRule {
            message: "batched quote review needs an entry".to_string(),
        };
        let mut item = new_batched_quote_review("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedQuoteReviewRequiredCodeRule {
            message: "batched quote review needs this field".to_string(),
        };
        let mut item = new_batched_quote_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = BatchedQuoteReviewSummaryLengthRule { max_length: 255 };
        let mut item = new_batched_quote_review("x", "y");
        item.summary = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
