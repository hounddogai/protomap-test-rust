use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast review must pass. Check returns its problems, or none.
pub trait ForecastReviewRule: std::fmt::Debug {
    fn check(&self, item: &ForecastReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ForecastReviewRule for ForecastReviewAllowedCodeRule {
    fn check(&self, item: &ForecastReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReviewRequiredCodeRule {
    pub message: String,
}

impl ForecastReviewRule for ForecastReviewRequiredCodeRule {
    fn check(&self, item: &ForecastReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReviewCodePrefixRule {
    pub prefix: String,
}

impl ForecastReviewRule for ForecastReviewCodePrefixRule {
    fn check(&self, item: &ForecastReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast review.
#[derive(Debug, Clone)]
pub struct ForecastReviewValidator {
    pub rules: Vec<Rc<dyn ForecastReviewRule>>,
}

impl ForecastReviewValidator {
    pub fn validate(&self, item: &ForecastReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_review_validator() -> ForecastReviewValidator {
    ForecastReviewValidator {
        rules: vec![
            Rc::new(ForecastReviewAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn ForecastReviewRule>,
            Rc::new(ForecastReviewRequiredCodeRule {
                message: "forecast review needs a value".to_string(),
            }),
            Rc::new(ForecastReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
