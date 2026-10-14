use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote budget must pass. Check returns its problems, or none.
pub trait QuoteBudgetRule: std::fmt::Debug {
    fn check(&self, item: &QuoteBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBudgetCodePrefixRule {
    pub prefix: String,
}

impl QuoteBudgetRule for QuoteBudgetCodePrefixRule {
    fn check(&self, item: &QuoteBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBudgetRequiredTitleRule {
    pub message: String,
}

impl QuoteBudgetRule for QuoteBudgetRequiredTitleRule {
    fn check(&self, item: &QuoteBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBudgetTitleLengthRule {
    pub max_length: i64,
}

impl QuoteBudgetRule for QuoteBudgetTitleLengthRule {
    fn check(&self, item: &QuoteBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBudgetGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteBudgetRule for QuoteBudgetGraceDaysRangeRule {
    fn check(&self, item: &QuoteBudget) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBudgetCodeLengthRule {
    pub max_length: i64,
}

impl QuoteBudgetRule for QuoteBudgetCodeLengthRule {
    fn check(&self, item: &QuoteBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote budget.
#[derive(Debug, Clone)]
pub struct QuoteBudgetValidator {
    pub rules: Vec<Rc<dyn QuoteBudgetRule>>,
}

impl QuoteBudgetValidator {
    pub fn validate(&self, item: &QuoteBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_budget_validator() -> QuoteBudgetValidator {
    QuoteBudgetValidator {
        rules: vec![
            Rc::new(QuoteBudgetCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn QuoteBudgetRule>,
            Rc::new(QuoteBudgetRequiredTitleRule {
                message: "quote budget needs a value".to_string(),
            }),
            Rc::new(QuoteBudgetTitleLengthRule { max_length: 80 }),
            Rc::new(QuoteBudgetGraceDaysRangeRule { min: 64, max: 120 }),
            Rc::new(QuoteBudgetCodeLengthRule { max_length: 16 }),
        ],
    }
}
