use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending verifier review must pass. Check returns its problems, or none.
pub trait PendingVerifierReviewRule: std::fmt::Debug {
    fn check(&self, item: &PendingVerifierReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierReviewAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingVerifierReviewRule for PendingVerifierReviewAllowedLabelRule {
    fn check(&self, item: &PendingVerifierReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierReviewTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingVerifierReviewRule for PendingVerifierReviewTtlSecondsRangeRule {
    fn check(&self, item: &PendingVerifierReview) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierReviewLabelLengthRule {
    pub max_length: i64,
}

impl PendingVerifierReviewRule for PendingVerifierReviewLabelLengthRule {
    fn check(&self, item: &PendingVerifierReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierReviewAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingVerifierReviewRule for PendingVerifierReviewAttemptsRangeRule {
    fn check(&self, item: &PendingVerifierReview) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierReviewRequiredFingerprintRule {
    pub message: String,
}

impl PendingVerifierReviewRule for PendingVerifierReviewRequiredFingerprintRule {
    fn check(&self, item: &PendingVerifierReview) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierReviewPrincipalLengthRule {
    pub max_length: i64,
}

impl PendingVerifierReviewRule for PendingVerifierReviewPrincipalLengthRule {
    fn check(&self, item: &PendingVerifierReview) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending verifier review.
#[derive(Debug, Clone)]
pub struct PendingVerifierReviewValidator {
    pub rules: Vec<Rc<dyn PendingVerifierReviewRule>>,
}

impl PendingVerifierReviewValidator {
    pub fn validate(&self, item: &PendingVerifierReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingVerifierReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingVerifierReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_verifier_review_validator() -> PendingVerifierReviewValidator {
    PendingVerifierReviewValidator {
        rules: vec![
            Rc::new(PendingVerifierReviewAllowedLabelRule {
                allowed: vec!["strict".to_string(), "weighted".to_string()],
            }) as Rc<dyn PendingVerifierReviewRule>,
            Rc::new(PendingVerifierReviewTtlSecondsRangeRule { min: 64, max: 64 }),
            Rc::new(PendingVerifierReviewLabelLengthRule { max_length: 86400 }),
            Rc::new(PendingVerifierReviewAttemptsRangeRule { min: 32, max: 86400 }),
            Rc::new(PendingVerifierReviewRequiredFingerprintRule {
                message: "pending verifier review needs a value".to_string(),
            }),
            Rc::new(PendingVerifierReviewPrincipalLengthRule { max_length: 1000 }),
        ],
    }
}
