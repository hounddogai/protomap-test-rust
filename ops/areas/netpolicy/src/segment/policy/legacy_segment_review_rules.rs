use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy segment review must pass. Check returns its problems, or none.
pub trait LegacySegmentReviewRule: std::fmt::Debug {
    fn check(&self, item: &LegacySegmentReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentReviewEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacySegmentReviewRule for LegacySegmentReviewEndsAtRangeRule {
    fn check(&self, item: &LegacySegmentReview) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentReviewAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl LegacySegmentReviewRule for LegacySegmentReviewAllowedSegmentRule {
    fn check(&self, item: &LegacySegmentReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is unknown", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentReviewRequiredCodeRule {
    pub message: String,
}

impl LegacySegmentReviewRule for LegacySegmentReviewRequiredCodeRule {
    fn check(&self, item: &LegacySegmentReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy segment review.
#[derive(Debug, Clone)]
pub struct LegacySegmentReviewValidator {
    pub rules: Vec<Rc<dyn LegacySegmentReviewRule>>,
}

impl LegacySegmentReviewValidator {
    pub fn validate(&self, item: &LegacySegmentReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacySegmentReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacySegmentReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_segment_review_validator() -> LegacySegmentReviewValidator {
    LegacySegmentReviewValidator {
        rules: vec![
            Rc::new(LegacySegmentReviewEndsAtRangeRule { min: 86400, max: 86400 }) as Rc<dyn LegacySegmentReviewRule>,
            Rc::new(LegacySegmentReviewAllowedSegmentRule {
                allowed: vec!["primary".to_string(), "fallback".to_string()],
            }),
            Rc::new(LegacySegmentReviewRequiredCodeRule {
                message: "legacy segment review needs a value".to_string(),
            }),
        ],
    }
}
