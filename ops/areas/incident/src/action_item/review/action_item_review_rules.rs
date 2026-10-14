use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item review must pass. Check returns its problems, or none.
pub trait ActionItemReviewRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemReviewRequiredCodeRule {
    pub message: String,
}

impl ActionItemReviewRule for ActionItemReviewRequiredCodeRule {
    fn check(&self, item: &ActionItemReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemReviewCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemReviewRule for ActionItemReviewCodeLengthRule {
    fn check(&self, item: &ActionItemReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemReviewCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemReviewRule for ActionItemReviewCostRangeRule {
    fn check(&self, item: &ActionItemReview) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemReviewStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemReviewRule for ActionItemReviewStartsAtRangeRule {
    fn check(&self, item: &ActionItemReview) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a action item review.
#[derive(Debug, Clone)]
pub struct ActionItemReviewValidator {
    pub rules: Vec<Rc<dyn ActionItemReviewRule>>,
}

impl ActionItemReviewValidator {
    pub fn validate(&self, item: &ActionItemReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_review_validator() -> ActionItemReviewValidator {
    ActionItemReviewValidator {
        rules: vec![
            Rc::new(ActionItemReviewRequiredCodeRule {
                message: "action item review needs an entry".to_string(),
            }) as Rc<dyn ActionItemReviewRule>,
            Rc::new(ActionItemReviewCodeLengthRule { max_length: 255 }),
            Rc::new(ActionItemReviewCostRangeRule { min: 2.0, max: 2283.7 }),
            Rc::new(ActionItemReviewStartsAtRangeRule { min: 32, max: 120 }),
        ],
    }
}
