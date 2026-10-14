use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink review must pass. Check returns its problems, or none.
pub trait SinkReviewRule: std::fmt::Debug {
    fn check(&self, item: &SinkReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReviewRequiredFingerprintRule {
    pub message: String,
}

impl SinkReviewRule for SinkReviewRequiredFingerprintRule {
    fn check(&self, item: &SinkReview) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReviewRequiredCodeRule {
    pub message: String,
}

impl SinkReviewRule for SinkReviewRequiredCodeRule {
    fn check(&self, item: &SinkReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReviewCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkReviewRule for SinkReviewCreatedAtRangeRule {
    fn check(&self, item: &SinkReview) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReviewNotesLengthRule {
    pub max_length: i64,
}

impl SinkReviewRule for SinkReviewNotesLengthRule {
    fn check(&self, item: &SinkReview) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReviewCodePrefixRule {
    pub prefix: String,
}

impl SinkReviewRule for SinkReviewCodePrefixRule {
    fn check(&self, item: &SinkReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink review.
#[derive(Debug, Clone)]
pub struct SinkReviewValidator {
    pub rules: Vec<Rc<dyn SinkReviewRule>>,
}

impl SinkReviewValidator {
    pub fn validate(&self, item: &SinkReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_review_validator() -> SinkReviewValidator {
    SinkReviewValidator {
        rules: vec![
            Rc::new(SinkReviewRequiredFingerprintRule {
                message: "sink review needs a value".to_string(),
            }) as Rc<dyn SinkReviewRule>,
            Rc::new(SinkReviewRequiredCodeRule {
                message: "sink review needs a value".to_string(),
            }),
            Rc::new(SinkReviewCreatedAtRangeRule { min: 32, max: 86400 }),
            Rc::new(SinkReviewNotesLengthRule { max_length: 120 }),
            Rc::new(SinkReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
