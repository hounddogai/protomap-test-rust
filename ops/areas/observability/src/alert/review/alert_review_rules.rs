use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert review must pass. Check returns its problems, or none.
pub trait AlertReviewRule: std::fmt::Debug {
    fn check(&self, item: &AlertReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertReviewCurrencyLengthRule {
    pub max_length: i64,
}

impl AlertReviewRule for AlertReviewCurrencyLengthRule {
    fn check(&self, item: &AlertReview) -> Vec<String> {
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
pub struct AlertReviewRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertReviewRule for AlertReviewRotationDaysRangeRule {
    fn check(&self, item: &AlertReview) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertReviewRequiredNameRule {
    pub message: String,
}

impl AlertReviewRule for AlertReviewRequiredNameRule {
    fn check(&self, item: &AlertReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert review.
#[derive(Debug, Clone)]
pub struct AlertReviewValidator {
    pub rules: Vec<Rc<dyn AlertReviewRule>>,
}

impl AlertReviewValidator {
    pub fn validate(&self, item: &AlertReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_review_validator() -> AlertReviewValidator {
    AlertReviewValidator {
        rules: vec![
            Rc::new(AlertReviewCurrencyLengthRule { max_length: 120 }) as Rc<dyn AlertReviewRule>,
            Rc::new(AlertReviewRotationDaysRangeRule { min: 16, max: 32 }),
            Rc::new(AlertReviewRequiredNameRule {
                message: "alert review needs an entry".to_string(),
            }),
        ],
    }
}
