use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling region pin review must pass. Check returns its problems, or none.
pub trait RollingRegionPinReviewRule: std::fmt::Debug {
    fn check(&self, item: &RollingRegionPinReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinReviewAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RollingRegionPinReviewRule for RollingRegionPinReviewAllowedTitleRule {
    fn check(&self, item: &RollingRegionPinReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinReviewThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingRegionPinReviewRule for RollingRegionPinReviewThresholdRangeRule {
    fn check(&self, item: &RollingRegionPinReview) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinReviewRequiredCipherRule {
    pub message: String,
}

impl RollingRegionPinReviewRule for RollingRegionPinReviewRequiredCipherRule {
    fn check(&self, item: &RollingRegionPinReview) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling region pin review.
#[derive(Debug, Clone)]
pub struct RollingRegionPinReviewValidator {
    pub rules: Vec<Rc<dyn RollingRegionPinReviewRule>>,
}

impl RollingRegionPinReviewValidator {
    pub fn validate(&self, item: &RollingRegionPinReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRegionPinReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRegionPinReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_region_pin_review_validator() -> RollingRegionPinReviewValidator {
    RollingRegionPinReviewValidator {
        rules: vec![
            Rc::new(RollingRegionPinReviewAllowedTitleRule {
                allowed: vec!["lenient".to_string(), "strict".to_string()],
            }) as Rc<dyn RollingRegionPinReviewRule>,
            Rc::new(RollingRegionPinReviewThresholdRangeRule { min: 6.5, max: 877.8 }),
            Rc::new(RollingRegionPinReviewRequiredCipherRule {
                message: "rolling region pin review needs this field".to_string(),
            }),
        ],
    }
}
