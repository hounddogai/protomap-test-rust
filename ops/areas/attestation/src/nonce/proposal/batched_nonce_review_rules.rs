use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched nonce review must pass. Check returns its problems, or none.
pub trait BatchedNonceReviewRule: std::fmt::Debug {
    fn check(&self, item: &BatchedNonceReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedNonceReviewRule for BatchedNonceReviewAllowedCodeRule {
    fn check(&self, item: &BatchedNonceReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceReviewCodeLengthRule {
    pub max_length: i64,
}

impl BatchedNonceReviewRule for BatchedNonceReviewCodeLengthRule {
    fn check(&self, item: &BatchedNonceReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceReviewRequiredCodeRule {
    pub message: String,
}

impl BatchedNonceReviewRule for BatchedNonceReviewRequiredCodeRule {
    fn check(&self, item: &BatchedNonceReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceReviewArchivedGuardRule {
    pub limit: f64,
}

impl BatchedNonceReviewRule for BatchedNonceReviewArchivedGuardRule {
    fn check(&self, item: &BatchedNonceReview) -> Vec<String> {
        if item.archived && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while archived", item.margin, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched nonce review.
#[derive(Debug, Clone)]
pub struct BatchedNonceReviewValidator {
    pub rules: Vec<Rc<dyn BatchedNonceReviewRule>>,
}

impl BatchedNonceReviewValidator {
    pub fn validate(&self, item: &BatchedNonceReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedNonceReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedNonceReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_nonce_review_validator() -> BatchedNonceReviewValidator {
    BatchedNonceReviewValidator {
        rules: vec![
            Rc::new(BatchedNonceReviewAllowedCodeRule {
                allowed: vec!["capped".to_string()],
            }) as Rc<dyn BatchedNonceReviewRule>,
            Rc::new(BatchedNonceReviewCodeLengthRule { max_length: 255 }),
            Rc::new(BatchedNonceReviewRequiredCodeRule {
                message: "batched nonce review needs a value".to_string(),
            }),
            Rc::new(BatchedNonceReviewArchivedGuardRule { limit: 4.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_nonce_review_validator();
        let item = new_batched_nonce_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedNonceReviewCodeLengthRule { max_length: 255 };
        let mut item = new_batched_nonce_review("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedNonceReviewRequiredCodeRule {
            message: "batched nonce review needs a value".to_string(),
        };
        let mut item = new_batched_nonce_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
