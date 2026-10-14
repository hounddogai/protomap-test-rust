use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier review must pass. Check returns its problems, or none.
pub trait VerifierReviewRule: std::fmt::Debug {
    fn check(&self, item: &VerifierReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReviewClusterLengthRule {
    pub max_length: i64,
}

impl VerifierReviewRule for VerifierReviewClusterLengthRule {
    fn check(&self, item: &VerifierReview) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReviewQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierReviewRule for VerifierReviewQuotaUnitsRangeRule {
    fn check(&self, item: &VerifierReview) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReviewRequiredClusterRule {
    pub message: String,
}

impl VerifierReviewRule for VerifierReviewRequiredClusterRule {
    fn check(&self, item: &VerifierReview) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReviewGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierReviewRule for VerifierReviewGraceDaysRangeRule {
    fn check(&self, item: &VerifierReview) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReviewContactEmailLengthRule {
    pub max_length: i64,
}

impl VerifierReviewRule for VerifierReviewContactEmailLengthRule {
    fn check(&self, item: &VerifierReview) -> Vec<String> {
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

/// Runs every rule of a verifier review.
#[derive(Debug, Clone)]
pub struct VerifierReviewValidator {
    pub rules: Vec<Rc<dyn VerifierReviewRule>>,
}

impl VerifierReviewValidator {
    pub fn validate(&self, item: &VerifierReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_review_validator() -> VerifierReviewValidator {
    VerifierReviewValidator {
        rules: vec![
            Rc::new(VerifierReviewClusterLengthRule { max_length: 32 }) as Rc<dyn VerifierReviewRule>,
            Rc::new(VerifierReviewQuotaUnitsRangeRule { min: 64, max: 64 }),
            Rc::new(VerifierReviewRequiredClusterRule {
                message: "verifier review needs this field".to_string(),
            }),
            Rc::new(VerifierReviewGraceDaysRangeRule { min: 16, max: 255 }),
            Rc::new(VerifierReviewContactEmailLengthRule { max_length: 3600 }),
        ],
    }
}
