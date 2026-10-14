use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention review must pass. Check returns its problems, or none.
pub trait RetentionReviewRule: std::fmt::Debug {
    fn check(&self, item: &RetentionReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReviewCodeLengthRule {
    pub max_length: i64,
}

impl RetentionReviewRule for RetentionReviewCodeLengthRule {
    fn check(&self, item: &RetentionReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReviewDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionReviewRule for RetentionReviewDiscountRangeRule {
    fn check(&self, item: &RetentionReview) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReviewRequiredPhoneRule {
    pub message: String,
}

impl RetentionReviewRule for RetentionReviewRequiredPhoneRule {
    fn check(&self, item: &RetentionReview) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReviewMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionReviewRule for RetentionReviewMarginRangeRule {
    fn check(&self, item: &RetentionReview) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention review.
#[derive(Debug, Clone)]
pub struct RetentionReviewValidator {
    pub rules: Vec<Rc<dyn RetentionReviewRule>>,
}

impl RetentionReviewValidator {
    pub fn validate(&self, item: &RetentionReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_review_validator() -> RetentionReviewValidator {
    RetentionReviewValidator {
        rules: vec![
            Rc::new(RetentionReviewCodeLengthRule { max_length: 3600 }) as Rc<dyn RetentionReviewRule>,
            Rc::new(RetentionReviewDiscountRangeRule { min: 6.7, max: 2126.6 }),
            Rc::new(RetentionReviewRequiredPhoneRule {
                message: "retention review needs this field".to_string(),
            }),
            Rc::new(RetentionReviewMarginRangeRule { min: 4.2, max: 874.8 }),
        ],
    }
}
