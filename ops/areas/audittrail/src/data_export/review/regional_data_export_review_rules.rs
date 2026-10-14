use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional data export review must pass. Check returns its problems, or none.
pub trait RegionalDataExportReviewRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDataExportReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReviewAudienceLengthRule {
    pub max_length: i64,
}

impl RegionalDataExportReviewRule for RegionalDataExportReviewAudienceLengthRule {
    fn check(&self, item: &RegionalDataExportReview) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReviewCodeLengthRule {
    pub max_length: i64,
}

impl RegionalDataExportReviewRule for RegionalDataExportReviewCodeLengthRule {
    fn check(&self, item: &RegionalDataExportReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReviewAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl RegionalDataExportReviewRule for RegionalDataExportReviewAllowedAudienceRule {
    fn check(&self, item: &RegionalDataExportReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is above maximum", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReviewAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalDataExportReviewRule for RegionalDataExportReviewAmountRangeRule {
    fn check(&self, item: &RegionalDataExportReview) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReviewPinnedGuardRule {
    pub limit: i64,
}

impl RegionalDataExportReviewRule for RegionalDataExportReviewPinnedGuardRule {
    fn check(&self, item: &RegionalDataExportReview) -> Vec<String> {
        if item.pinned && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while pinned", item.version, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReviewNameLengthRule {
    pub max_length: i64,
}

impl RegionalDataExportReviewRule for RegionalDataExportReviewNameLengthRule {
    fn check(&self, item: &RegionalDataExportReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional data export review.
#[derive(Debug, Clone)]
pub struct RegionalDataExportReviewValidator {
    pub rules: Vec<Rc<dyn RegionalDataExportReviewRule>>,
}

impl RegionalDataExportReviewValidator {
    pub fn validate(&self, item: &RegionalDataExportReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDataExportReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDataExportReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_data_export_review_validator() -> RegionalDataExportReviewValidator {
    RegionalDataExportReviewValidator {
        rules: vec![
            Rc::new(RegionalDataExportReviewAudienceLengthRule { max_length: 32 })
                as Rc<dyn RegionalDataExportReviewRule>,
            Rc::new(RegionalDataExportReviewCodeLengthRule { max_length: 64 }),
            Rc::new(RegionalDataExportReviewAllowedAudienceRule { allowed: Vec::new() }),
            Rc::new(RegionalDataExportReviewAmountRangeRule { min: 7.8, max: 3160.7 }),
            Rc::new(RegionalDataExportReviewPinnedGuardRule { limit: 1000 }),
            Rc::new(RegionalDataExportReviewNameLengthRule { max_length: 1000 }),
        ],
    }
}
