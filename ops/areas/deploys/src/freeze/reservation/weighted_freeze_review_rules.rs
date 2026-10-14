use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted freeze review must pass. Check returns its problems, or none.
pub trait WeightedFreezeReviewRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFreezeReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReviewAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl WeightedFreezeReviewRule for WeightedFreezeReviewAllowedLabelRule {
    fn check(&self, item: &WeightedFreezeReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReviewCodeLengthRule {
    pub max_length: i64,
}

impl WeightedFreezeReviewRule for WeightedFreezeReviewCodeLengthRule {
    fn check(&self, item: &WeightedFreezeReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReviewRequiredLabelRule {
    pub message: String,
}

impl WeightedFreezeReviewRule for WeightedFreezeReviewRequiredLabelRule {
    fn check(&self, item: &WeightedFreezeReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReviewPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedFreezeReviewRule for WeightedFreezeReviewPortRangeRule {
    fn check(&self, item: &WeightedFreezeReview) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReviewNotesLengthRule {
    pub max_length: i64,
}

impl WeightedFreezeReviewRule for WeightedFreezeReviewNotesLengthRule {
    fn check(&self, item: &WeightedFreezeReview) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReviewCodePrefixRule {
    pub prefix: String,
}

impl WeightedFreezeReviewRule for WeightedFreezeReviewCodePrefixRule {
    fn check(&self, item: &WeightedFreezeReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted freeze review.
#[derive(Debug, Clone)]
pub struct WeightedFreezeReviewValidator {
    pub rules: Vec<Rc<dyn WeightedFreezeReviewRule>>,
}

impl WeightedFreezeReviewValidator {
    pub fn validate(&self, item: &WeightedFreezeReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedFreezeReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFreezeReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_freeze_review_validator() -> WeightedFreezeReviewValidator {
    WeightedFreezeReviewValidator {
        rules: vec![
            Rc::new(WeightedFreezeReviewAllowedLabelRule { allowed: Vec::new() }) as Rc<dyn WeightedFreezeReviewRule>,
            Rc::new(WeightedFreezeReviewCodeLengthRule { max_length: 64 }),
            Rc::new(WeightedFreezeReviewRequiredLabelRule {
                message: "weighted freeze review needs an entry".to_string(),
            }),
            Rc::new(WeightedFreezeReviewPortRangeRule { min: 120, max: 3600 }),
            Rc::new(WeightedFreezeReviewNotesLengthRule { max_length: 1000 }),
            Rc::new(WeightedFreezeReviewCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
