use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem review must pass. Check returns its problems, or none.
pub trait PostmortemReviewRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReviewRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemReviewRule for PostmortemReviewRevisionRangeRule {
    fn check(&self, item: &PostmortemReview) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReviewCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemReviewRule for PostmortemReviewCodeLengthRule {
    fn check(&self, item: &PostmortemReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReviewRequiredCodeRule {
    pub message: String,
}

impl PostmortemReviewRule for PostmortemReviewRequiredCodeRule {
    fn check(&self, item: &PostmortemReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem review.
#[derive(Debug, Clone)]
pub struct PostmortemReviewValidator {
    pub rules: Vec<Rc<dyn PostmortemReviewRule>>,
}

impl PostmortemReviewValidator {
    pub fn validate(&self, item: &PostmortemReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_review_validator() -> PostmortemReviewValidator {
    PostmortemReviewValidator {
        rules: vec![
            Rc::new(PostmortemReviewRevisionRangeRule { min: 64, max: 80 }) as Rc<dyn PostmortemReviewRule>,
            Rc::new(PostmortemReviewCodeLengthRule { max_length: 255 }),
            Rc::new(PostmortemReviewRequiredCodeRule {
                message: "postmortem review needs a value".to_string(),
            }),
        ],
    }
}
