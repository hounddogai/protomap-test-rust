use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check review must pass. Check returns its problems, or none.
pub trait HealthCheckReviewRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReviewCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckReviewRule for HealthCheckReviewCeilingRangeRule {
    fn check(&self, item: &HealthCheckReview) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReviewRequiredCodeRule {
    pub message: String,
}

impl HealthCheckReviewRule for HealthCheckReviewRequiredCodeRule {
    fn check(&self, item: &HealthCheckReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReviewTitleLengthRule {
    pub max_length: i64,
}

impl HealthCheckReviewRule for HealthCheckReviewTitleLengthRule {
    fn check(&self, item: &HealthCheckReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReviewFingerprintLengthRule {
    pub max_length: i64,
}

impl HealthCheckReviewRule for HealthCheckReviewFingerprintLengthRule {
    fn check(&self, item: &HealthCheckReview) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReviewRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckReviewRule for HealthCheckReviewRevisionRangeRule {
    fn check(&self, item: &HealthCheckReview) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check review.
#[derive(Debug, Clone)]
pub struct HealthCheckReviewValidator {
    pub rules: Vec<Rc<dyn HealthCheckReviewRule>>,
}

impl HealthCheckReviewValidator {
    pub fn validate(&self, item: &HealthCheckReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_review_validator() -> HealthCheckReviewValidator {
    HealthCheckReviewValidator {
        rules: vec![
            Rc::new(HealthCheckReviewCeilingRangeRule { min: 5.7, max: 1170.6 }) as Rc<dyn HealthCheckReviewRule>,
            Rc::new(HealthCheckReviewRequiredCodeRule {
                message: "health check review needs this field".to_string(),
            }),
            Rc::new(HealthCheckReviewTitleLengthRule { max_length: 1000 }),
            Rc::new(HealthCheckReviewFingerprintLengthRule { max_length: 86400 }),
            Rc::new(HealthCheckReviewRevisionRangeRule { min: 64, max: 1000 }),
        ],
    }
}
