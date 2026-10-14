use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending postmortem review must pass. Check returns its problems, or none.
pub trait PendingPostmortemReviewRule: std::fmt::Debug {
    fn check(&self, item: &PendingPostmortemReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemReviewDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingPostmortemReviewRule for PendingPostmortemReviewDurationMinutesRangeRule {
    fn check(&self, item: &PendingPostmortemReview) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemReviewExternalRefLengthRule {
    pub max_length: i64,
}

impl PendingPostmortemReviewRule for PendingPostmortemReviewExternalRefLengthRule {
    fn check(&self, item: &PendingPostmortemReview) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemReviewCodePrefixRule {
    pub prefix: String,
}

impl PendingPostmortemReviewRule for PendingPostmortemReviewCodePrefixRule {
    fn check(&self, item: &PendingPostmortemReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemReviewAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl PendingPostmortemReviewRule for PendingPostmortemReviewAllowedExternalRefRule {
    fn check(&self, item: &PendingPostmortemReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is missing", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemReviewRequiredCodeRule {
    pub message: String,
}

impl PendingPostmortemReviewRule for PendingPostmortemReviewRequiredCodeRule {
    fn check(&self, item: &PendingPostmortemReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemReviewCurrencyLengthRule {
    pub max_length: i64,
}

impl PendingPostmortemReviewRule for PendingPostmortemReviewCurrencyLengthRule {
    fn check(&self, item: &PendingPostmortemReview) -> Vec<String> {
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

/// Runs every rule of a pending postmortem review.
#[derive(Debug, Clone)]
pub struct PendingPostmortemReviewValidator {
    pub rules: Vec<Rc<dyn PendingPostmortemReviewRule>>,
}

impl PendingPostmortemReviewValidator {
    pub fn validate(&self, item: &PendingPostmortemReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPostmortemReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPostmortemReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_postmortem_review_validator() -> PendingPostmortemReviewValidator {
    PendingPostmortemReviewValidator {
        rules: vec![
            Rc::new(PendingPostmortemReviewDurationMinutesRangeRule { min: 86400, max: 86400 })
                as Rc<dyn PendingPostmortemReviewRule>,
            Rc::new(PendingPostmortemReviewExternalRefLengthRule { max_length: 64 }),
            Rc::new(PendingPostmortemReviewCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingPostmortemReviewAllowedExternalRefRule { allowed: Vec::new() }),
            Rc::new(PendingPostmortemReviewRequiredCodeRule {
                message: "pending postmortem review needs this field".to_string(),
            }),
            Rc::new(PendingPostmortemReviewCurrencyLengthRule { max_length: 255 }),
        ],
    }
}
