use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal rollback review must pass. Check returns its problems, or none.
pub trait SeasonalRollbackReviewRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRollbackReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackReviewLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalRollbackReviewRule for SeasonalRollbackReviewLabelLengthRule {
    fn check(&self, item: &SeasonalRollbackReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackReviewTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl SeasonalRollbackReviewRule for SeasonalRollbackReviewTagsLimitRule {
    fn check(&self, item: &SeasonalRollbackReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackReviewScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalRollbackReviewRule for SeasonalRollbackReviewScoreRangeRule {
    fn check(&self, item: &SeasonalRollbackReview) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackReviewExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRollbackReviewRule for SeasonalRollbackReviewExpiresAtRangeRule {
    fn check(&self, item: &SeasonalRollbackReview) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackReviewRequiredCodeRule {
    pub message: String,
}

impl SeasonalRollbackReviewRule for SeasonalRollbackReviewRequiredCodeRule {
    fn check(&self, item: &SeasonalRollbackReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal rollback review.
#[derive(Debug, Clone)]
pub struct SeasonalRollbackReviewValidator {
    pub rules: Vec<Rc<dyn SeasonalRollbackReviewRule>>,
}

impl SeasonalRollbackReviewValidator {
    pub fn validate(&self, item: &SeasonalRollbackReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalRollbackReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRollbackReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_rollback_review_validator() -> SeasonalRollbackReviewValidator {
    SeasonalRollbackReviewValidator {
        rules: vec![
            Rc::new(SeasonalRollbackReviewLabelLengthRule { max_length: 64 }) as Rc<dyn SeasonalRollbackReviewRule>,
            Rc::new(SeasonalRollbackReviewTagsLimitRule {
                max_count: 64,
                banned: vec!["rolling".to_string(), "regional".to_string(), "default".to_string()],
            }),
            Rc::new(SeasonalRollbackReviewScoreRangeRule { min: 9.2, max: 2375.9 }),
            Rc::new(SeasonalRollbackReviewExpiresAtRangeRule { min: 16, max: 255 }),
            Rc::new(SeasonalRollbackReviewRequiredCodeRule {
                message: "seasonal rollback review needs a value".to_string(),
            }),
        ],
    }
}
