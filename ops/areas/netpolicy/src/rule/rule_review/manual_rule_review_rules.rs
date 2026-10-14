use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual rule review must pass. Check returns its problems, or none.
pub trait ManualRuleReviewRule: std::fmt::Debug {
    fn check(&self, item: &ManualRuleReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleReviewRegionLengthRule {
    pub max_length: i64,
}

impl ManualRuleReviewRule for ManualRuleReviewRegionLengthRule {
    fn check(&self, item: &ManualRuleReview) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleReviewCodeLengthRule {
    pub max_length: i64,
}

impl ManualRuleReviewRule for ManualRuleReviewCodeLengthRule {
    fn check(&self, item: &ManualRuleReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleReviewRequiredLabelRule {
    pub message: String,
}

impl ManualRuleReviewRule for ManualRuleReviewRequiredLabelRule {
    fn check(&self, item: &ManualRuleReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleReviewAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl ManualRuleReviewRule for ManualRuleReviewAllowedRegionRule {
    fn check(&self, item: &ManualRuleReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is expired", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual rule review.
#[derive(Debug, Clone)]
pub struct ManualRuleReviewValidator {
    pub rules: Vec<Rc<dyn ManualRuleReviewRule>>,
}

impl ManualRuleReviewValidator {
    pub fn validate(&self, item: &ManualRuleReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualRuleReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRuleReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_rule_review_validator() -> ManualRuleReviewValidator {
    ManualRuleReviewValidator {
        rules: vec![
            Rc::new(ManualRuleReviewRegionLengthRule { max_length: 32 }) as Rc<dyn ManualRuleReviewRule>,
            Rc::new(ManualRuleReviewCodeLengthRule { max_length: 1000 }),
            Rc::new(ManualRuleReviewRequiredLabelRule {
                message: "manual rule review needs a value".to_string(),
            }),
            Rc::new(ManualRuleReviewAllowedRegionRule { allowed: Vec::new() }),
        ],
    }
}
