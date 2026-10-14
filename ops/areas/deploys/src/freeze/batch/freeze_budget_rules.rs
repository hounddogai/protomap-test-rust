use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze budget must pass. Check returns its problems, or none.
pub trait FreezeBudgetRule: std::fmt::Debug {
    fn check(&self, item: &FreezeBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBudgetRequiredTitleRule {
    pub message: String,
}

impl FreezeBudgetRule for FreezeBudgetRequiredTitleRule {
    fn check(&self, item: &FreezeBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBudgetRequiredCodeRule {
    pub message: String,
}

impl FreezeBudgetRule for FreezeBudgetRequiredCodeRule {
    fn check(&self, item: &FreezeBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBudgetMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeBudgetRule for FreezeBudgetMaximumRangeRule {
    fn check(&self, item: &FreezeBudget) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBudgetTitleLengthRule {
    pub max_length: i64,
}

impl FreezeBudgetRule for FreezeBudgetTitleLengthRule {
    fn check(&self, item: &FreezeBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBudgetThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeBudgetRule for FreezeBudgetThresholdRangeRule {
    fn check(&self, item: &FreezeBudget) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBudgetCodeLengthRule {
    pub max_length: i64,
}

impl FreezeBudgetRule for FreezeBudgetCodeLengthRule {
    fn check(&self, item: &FreezeBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze budget.
#[derive(Debug, Clone)]
pub struct FreezeBudgetValidator {
    pub rules: Vec<Rc<dyn FreezeBudgetRule>>,
}

impl FreezeBudgetValidator {
    pub fn validate(&self, item: &FreezeBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_budget_validator() -> FreezeBudgetValidator {
    FreezeBudgetValidator {
        rules: vec![
            Rc::new(FreezeBudgetRequiredTitleRule {
                message: "freeze budget needs this field".to_string(),
            }) as Rc<dyn FreezeBudgetRule>,
            Rc::new(FreezeBudgetRequiredCodeRule {
                message: "freeze budget needs a value".to_string(),
            }),
            Rc::new(FreezeBudgetMaximumRangeRule { min: 2.3, max: 1323.3 }),
            Rc::new(FreezeBudgetTitleLengthRule { max_length: 255 }),
            Rc::new(FreezeBudgetThresholdRangeRule { min: 5.6, max: 2970.0 }),
            Rc::new(FreezeBudgetCodeLengthRule { max_length: 255 }),
        ],
    }
}
