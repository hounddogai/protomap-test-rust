use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted timeline review must pass. Check returns its problems, or none.
pub trait WeightedTimelineReviewRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTimelineReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReviewRequiredCodeRule {
    pub message: String,
}

impl WeightedTimelineReviewRule for WeightedTimelineReviewRequiredCodeRule {
    fn check(&self, item: &WeightedTimelineReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReviewCodePrefixRule {
    pub prefix: String,
}

impl WeightedTimelineReviewRule for WeightedTimelineReviewCodePrefixRule {
    fn check(&self, item: &WeightedTimelineReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReviewCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedTimelineReviewRule for WeightedTimelineReviewCeilingRangeRule {
    fn check(&self, item: &WeightedTimelineReview) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReviewCodeLengthRule {
    pub max_length: i64,
}

impl WeightedTimelineReviewRule for WeightedTimelineReviewCodeLengthRule {
    fn check(&self, item: &WeightedTimelineReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReviewQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedTimelineReviewRule for WeightedTimelineReviewQuantityRangeRule {
    fn check(&self, item: &WeightedTimelineReview) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReviewRequiredLabelRule {
    pub message: String,
}

impl WeightedTimelineReviewRule for WeightedTimelineReviewRequiredLabelRule {
    fn check(&self, item: &WeightedTimelineReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted timeline review.
#[derive(Debug, Clone)]
pub struct WeightedTimelineReviewValidator {
    pub rules: Vec<Rc<dyn WeightedTimelineReviewRule>>,
}

impl WeightedTimelineReviewValidator {
    pub fn validate(&self, item: &WeightedTimelineReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedTimelineReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTimelineReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_timeline_review_validator() -> WeightedTimelineReviewValidator {
    WeightedTimelineReviewValidator {
        rules: vec![
            Rc::new(WeightedTimelineReviewRequiredCodeRule {
                message: "weighted timeline review needs a value".to_string(),
            }) as Rc<dyn WeightedTimelineReviewRule>,
            Rc::new(WeightedTimelineReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedTimelineReviewCeilingRangeRule { min: 3.0, max: 3769.6 }),
            Rc::new(WeightedTimelineReviewCodeLengthRule { max_length: 32 }),
            Rc::new(WeightedTimelineReviewQuantityRangeRule { min: 32, max: 255 }),
            Rc::new(WeightedTimelineReviewRequiredLabelRule {
                message: "weighted timeline review needs this field".to_string(),
            }),
        ],
    }
}
