use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary failover review must pass. Check returns its problems, or none.
pub trait PrimaryFailoverReviewRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFailoverReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverReviewRequiredTitleRule {
    pub message: String,
}

impl PrimaryFailoverReviewRule for PrimaryFailoverReviewRequiredTitleRule {
    fn check(&self, item: &PrimaryFailoverReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverReviewRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryFailoverReviewRule for PrimaryFailoverReviewRatioRangeRule {
    fn check(&self, item: &PrimaryFailoverReview) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverReviewTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverReviewRule for PrimaryFailoverReviewTitleLengthRule {
    fn check(&self, item: &PrimaryFailoverReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverReviewRequiredHostRule {
    pub message: String,
}

impl PrimaryFailoverReviewRule for PrimaryFailoverReviewRequiredHostRule {
    fn check(&self, item: &PrimaryFailoverReview) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary failover review.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverReviewValidator {
    pub rules: Vec<Rc<dyn PrimaryFailoverReviewRule>>,
}

impl PrimaryFailoverReviewValidator {
    pub fn validate(&self, item: &PrimaryFailoverReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryFailoverReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFailoverReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_failover_review_validator() -> PrimaryFailoverReviewValidator {
    PrimaryFailoverReviewValidator {
        rules: vec![
            Rc::new(PrimaryFailoverReviewRequiredTitleRule {
                message: "primary failover review needs this field".to_string(),
            }) as Rc<dyn PrimaryFailoverReviewRule>,
            Rc::new(PrimaryFailoverReviewRatioRangeRule { min: 1.6, max: 1150.6 }),
            Rc::new(PrimaryFailoverReviewTitleLengthRule { max_length: 16 }),
            Rc::new(PrimaryFailoverReviewRequiredHostRule {
                message: "primary failover review needs this field".to_string(),
            }),
        ],
    }
}
