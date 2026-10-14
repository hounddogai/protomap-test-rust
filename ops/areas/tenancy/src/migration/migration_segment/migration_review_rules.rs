use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration review must pass. Check returns its problems, or none.
pub trait MigrationReviewRule: std::fmt::Debug {
    fn check(&self, item: &MigrationReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReviewLabelLengthRule {
    pub max_length: i64,
}

impl MigrationReviewRule for MigrationReviewLabelLengthRule {
    fn check(&self, item: &MigrationReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReviewRequiredSummaryRule {
    pub message: String,
}

impl MigrationReviewRule for MigrationReviewRequiredSummaryRule {
    fn check(&self, item: &MigrationReview) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReviewAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl MigrationReviewRule for MigrationReviewAllowedLabelRule {
    fn check(&self, item: &MigrationReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReviewRequiredLabelRule {
    pub message: String,
}

impl MigrationReviewRule for MigrationReviewRequiredLabelRule {
    fn check(&self, item: &MigrationReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration review.
#[derive(Debug, Clone)]
pub struct MigrationReviewValidator {
    pub rules: Vec<Rc<dyn MigrationReviewRule>>,
}

impl MigrationReviewValidator {
    pub fn validate(&self, item: &MigrationReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_review_validator() -> MigrationReviewValidator {
    MigrationReviewValidator {
        rules: vec![
            Rc::new(MigrationReviewLabelLengthRule { max_length: 1000 }) as Rc<dyn MigrationReviewRule>,
            Rc::new(MigrationReviewRequiredSummaryRule {
                message: "migration review needs this field".to_string(),
            }),
            Rc::new(MigrationReviewAllowedLabelRule {
                allowed: vec!["manual".to_string(), "default".to_string()],
            }),
            Rc::new(MigrationReviewRequiredLabelRule {
                message: "migration review needs this field".to_string(),
            }),
        ],
    }
}
