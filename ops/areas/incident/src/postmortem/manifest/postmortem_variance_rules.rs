use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem variance must pass. Check returns its problems, or none.
pub trait PostmortemVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemVarianceLabelLengthRule {
    pub max_length: i64,
}

impl PostmortemVarianceRule for PostmortemVarianceLabelLengthRule {
    fn check(&self, item: &PostmortemVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemVarianceAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemVarianceRule for PostmortemVarianceAmountRangeRule {
    fn check(&self, item: &PostmortemVariance) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemVarianceRequiredCategoryRule {
    pub message: String,
}

impl PostmortemVarianceRule for PostmortemVarianceRequiredCategoryRule {
    fn check(&self, item: &PostmortemVariance) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemVarianceLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemVarianceRule for PostmortemVarianceLimitRangeRule {
    fn check(&self, item: &PostmortemVariance) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemVarianceRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemVarianceRule for PostmortemVarianceRateRangeRule {
    fn check(&self, item: &PostmortemVariance) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem variance.
#[derive(Debug, Clone)]
pub struct PostmortemVarianceValidator {
    pub rules: Vec<Rc<dyn PostmortemVarianceRule>>,
}

impl PostmortemVarianceValidator {
    pub fn validate(&self, item: &PostmortemVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_variance_validator() -> PostmortemVarianceValidator {
    PostmortemVarianceValidator {
        rules: vec![
            Rc::new(PostmortemVarianceLabelLengthRule { max_length: 80 }) as Rc<dyn PostmortemVarianceRule>,
            Rc::new(PostmortemVarianceAmountRangeRule { min: 2.9, max: 498.8 }),
            Rc::new(PostmortemVarianceRequiredCategoryRule {
                message: "postmortem variance needs an entry".to_string(),
            }),
            Rc::new(PostmortemVarianceLimitRangeRule { min: 64, max: 255 }),
            Rc::new(PostmortemVarianceRateRangeRule { min: 5.4, max: 4244.9 }),
        ],
    }
}
