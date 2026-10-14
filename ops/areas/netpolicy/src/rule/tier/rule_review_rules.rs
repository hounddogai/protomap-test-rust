use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule review must pass. Check returns its problems, or none.
pub trait RuleReviewRule: std::fmt::Debug {
    fn check(&self, item: &RuleReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReviewFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleReviewRule for RuleReviewFloorValueRangeRule {
    fn check(&self, item: &RuleReview) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReviewAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl RuleReviewRule for RuleReviewAllowedAudienceRule {
    fn check(&self, item: &RuleReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is too long", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReviewRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleReviewRule for RuleReviewRateRangeRule {
    fn check(&self, item: &RuleReview) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReviewRequiredCodeRule {
    pub message: String,
}

impl RuleReviewRule for RuleReviewRequiredCodeRule {
    fn check(&self, item: &RuleReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReviewNameLengthRule {
    pub max_length: i64,
}

impl RuleReviewRule for RuleReviewNameLengthRule {
    fn check(&self, item: &RuleReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleReviewRequiredChecksumRule {
    pub message: String,
}

impl RuleReviewRule for RuleReviewRequiredChecksumRule {
    fn check(&self, item: &RuleReview) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule review.
#[derive(Debug, Clone)]
pub struct RuleReviewValidator {
    pub rules: Vec<Rc<dyn RuleReviewRule>>,
}

impl RuleReviewValidator {
    pub fn validate(&self, item: &RuleReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_review_validator() -> RuleReviewValidator {
    RuleReviewValidator {
        rules: vec![
            Rc::new(RuleReviewFloorValueRangeRule { min: 1.3, max: 2192.4 }) as Rc<dyn RuleReviewRule>,
            Rc::new(RuleReviewAllowedAudienceRule { allowed: Vec::new() }),
            Rc::new(RuleReviewRateRangeRule { min: 0.3, max: 2930.5 }),
            Rc::new(RuleReviewRequiredCodeRule {
                message: "rule review needs a value".to_string(),
            }),
            Rc::new(RuleReviewNameLengthRule { max_length: 255 }),
            Rc::new(RuleReviewRequiredChecksumRule {
                message: "rule review needs a value".to_string(),
            }),
        ],
    }
}
