use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence review must pass. Check returns its problems, or none.
pub trait EvidenceReviewRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReviewNameLengthRule {
    pub max_length: i64,
}

impl EvidenceReviewRule for EvidenceReviewNameLengthRule {
    fn check(&self, item: &EvidenceReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReviewTitleLengthRule {
    pub max_length: i64,
}

impl EvidenceReviewRule for EvidenceReviewTitleLengthRule {
    fn check(&self, item: &EvidenceReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReviewRequiredSegmentRule {
    pub message: String,
}

impl EvidenceReviewRule for EvidenceReviewRequiredSegmentRule {
    fn check(&self, item: &EvidenceReview) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReviewCodePrefixRule {
    pub prefix: String,
}

impl EvidenceReviewRule for EvidenceReviewCodePrefixRule {
    fn check(&self, item: &EvidenceReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReviewGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceReviewRule for EvidenceReviewGraceDaysRangeRule {
    fn check(&self, item: &EvidenceReview) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReviewSegmentLengthRule {
    pub max_length: i64,
}

impl EvidenceReviewRule for EvidenceReviewSegmentLengthRule {
    fn check(&self, item: &EvidenceReview) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence review.
#[derive(Debug, Clone)]
pub struct EvidenceReviewValidator {
    pub rules: Vec<Rc<dyn EvidenceReviewRule>>,
}

impl EvidenceReviewValidator {
    pub fn validate(&self, item: &EvidenceReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_review_validator() -> EvidenceReviewValidator {
    EvidenceReviewValidator {
        rules: vec![
            Rc::new(EvidenceReviewNameLengthRule { max_length: 3600 }) as Rc<dyn EvidenceReviewRule>,
            Rc::new(EvidenceReviewTitleLengthRule { max_length: 32 }),
            Rc::new(EvidenceReviewRequiredSegmentRule {
                message: "evidence review needs a value".to_string(),
            }),
            Rc::new(EvidenceReviewCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(EvidenceReviewGraceDaysRangeRule { min: 32, max: 80 }),
            Rc::new(EvidenceReviewSegmentLengthRule { max_length: 32 }),
        ],
    }
}
