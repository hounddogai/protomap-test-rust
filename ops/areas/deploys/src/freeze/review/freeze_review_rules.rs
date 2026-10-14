use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze review must pass. Check returns its problems, or none.
pub trait FreezeReviewRule: std::fmt::Debug {
    fn check(&self, item: &FreezeReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReviewCurrencyLengthRule {
    pub max_length: i64,
}

impl FreezeReviewRule for FreezeReviewCurrencyLengthRule {
    fn check(&self, item: &FreezeReview) -> Vec<String> {
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
pub struct FreezeReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FreezeReviewRule for FreezeReviewAllowedCodeRule {
    fn check(&self, item: &FreezeReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReviewBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeReviewRule for FreezeReviewBatchSizeRangeRule {
    fn check(&self, item: &FreezeReview) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze review.
#[derive(Debug, Clone)]
pub struct FreezeReviewValidator {
    pub rules: Vec<Rc<dyn FreezeReviewRule>>,
}

impl FreezeReviewValidator {
    pub fn validate(&self, item: &FreezeReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_review_validator() -> FreezeReviewValidator {
    FreezeReviewValidator {
        rules: vec![
            Rc::new(FreezeReviewCurrencyLengthRule { max_length: 64 }) as Rc<dyn FreezeReviewRule>,
            Rc::new(FreezeReviewAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "tiered".to_string(), "default".to_string()],
            }),
            Rc::new(FreezeReviewBatchSizeRangeRule { min: 64, max: 86400 }),
        ],
    }
}
