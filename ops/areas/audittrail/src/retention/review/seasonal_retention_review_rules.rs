use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal retention review must pass. Check returns its problems, or none.
pub trait SeasonalRetentionReviewRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRetentionReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionReviewCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalRetentionReviewRule for SeasonalRetentionReviewCodeLengthRule {
    fn check(&self, item: &SeasonalRetentionReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionReviewPrincipalLengthRule {
    pub max_length: i64,
}

impl SeasonalRetentionReviewRule for SeasonalRetentionReviewPrincipalLengthRule {
    fn check(&self, item: &SeasonalRetentionReview) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionReviewRequiredExternalRefRule {
    pub message: String,
}

impl SeasonalRetentionReviewRule for SeasonalRetentionReviewRequiredExternalRefRule {
    fn check(&self, item: &SeasonalRetentionReview) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal retention review.
#[derive(Debug, Clone)]
pub struct SeasonalRetentionReviewValidator {
    pub rules: Vec<Rc<dyn SeasonalRetentionReviewRule>>,
}

impl SeasonalRetentionReviewValidator {
    pub fn validate(&self, item: &SeasonalRetentionReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalRetentionReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRetentionReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_retention_review_validator() -> SeasonalRetentionReviewValidator {
    SeasonalRetentionReviewValidator {
        rules: vec![
            Rc::new(SeasonalRetentionReviewCodeLengthRule { max_length: 86400 }) as Rc<dyn SeasonalRetentionReviewRule>,
            Rc::new(SeasonalRetentionReviewPrincipalLengthRule { max_length: 86400 }),
            Rc::new(SeasonalRetentionReviewRequiredExternalRefRule {
                message: "seasonal retention review needs a value".to_string(),
            }),
        ],
    }
}
