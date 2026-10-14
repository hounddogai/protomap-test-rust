use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink budget must pass. Check returns its problems, or none.
pub trait SinkBudgetRule: std::fmt::Debug {
    fn check(&self, item: &SinkBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetReviewerLengthRule {
    pub max_length: i64,
}

impl SinkBudgetRule for SinkBudgetReviewerLengthRule {
    fn check(&self, item: &SinkBudget) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetNameLengthRule {
    pub max_length: i64,
}

impl SinkBudgetRule for SinkBudgetNameLengthRule {
    fn check(&self, item: &SinkBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetCategoryLengthRule {
    pub max_length: i64,
}

impl SinkBudgetRule for SinkBudgetCategoryLengthRule {
    fn check(&self, item: &SinkBudget) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetRequiredNameRule {
    pub message: String,
}

impl SinkBudgetRule for SinkBudgetRequiredNameRule {
    fn check(&self, item: &SinkBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkBudgetRule for SinkBudgetUpdatedAtRangeRule {
    fn check(&self, item: &SinkBudget) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkBudgetRule for SinkBudgetLimitRangeRule {
    fn check(&self, item: &SinkBudget) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink budget.
#[derive(Debug, Clone)]
pub struct SinkBudgetValidator {
    pub rules: Vec<Rc<dyn SinkBudgetRule>>,
}

impl SinkBudgetValidator {
    pub fn validate(&self, item: &SinkBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_budget_validator() -> SinkBudgetValidator {
    SinkBudgetValidator {
        rules: vec![
            Rc::new(SinkBudgetReviewerLengthRule { max_length: 16 }) as Rc<dyn SinkBudgetRule>,
            Rc::new(SinkBudgetNameLengthRule { max_length: 255 }),
            Rc::new(SinkBudgetCategoryLengthRule { max_length: 32 }),
            Rc::new(SinkBudgetRequiredNameRule {
                message: "sink budget needs this field".to_string(),
            }),
            Rc::new(SinkBudgetUpdatedAtRangeRule { min: 64, max: 80 }),
            Rc::new(SinkBudgetLimitRangeRule { min: 16, max: 80 }),
        ],
    }
}
