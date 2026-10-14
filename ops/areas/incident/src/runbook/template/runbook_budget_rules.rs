use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook budget must pass. Check returns its problems, or none.
pub trait RunbookBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RunbookBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBudgetTitleLengthRule {
    pub max_length: i64,
}

impl RunbookBudgetRule for RunbookBudgetTitleLengthRule {
    fn check(&self, item: &RunbookBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBudgetRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookBudgetRule for RunbookBudgetRevisionRangeRule {
    fn check(&self, item: &RunbookBudget) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBudgetKeyIdLengthRule {
    pub max_length: i64,
}

impl RunbookBudgetRule for RunbookBudgetKeyIdLengthRule {
    fn check(&self, item: &RunbookBudget) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBudgetRequiredTitleRule {
    pub message: String,
}

impl RunbookBudgetRule for RunbookBudgetRequiredTitleRule {
    fn check(&self, item: &RunbookBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook budget.
#[derive(Debug, Clone)]
pub struct RunbookBudgetValidator {
    pub rules: Vec<Rc<dyn RunbookBudgetRule>>,
}

impl RunbookBudgetValidator {
    pub fn validate(&self, item: &RunbookBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_budget_validator() -> RunbookBudgetValidator {
    RunbookBudgetValidator {
        rules: vec![
            Rc::new(RunbookBudgetTitleLengthRule { max_length: 32 }) as Rc<dyn RunbookBudgetRule>,
            Rc::new(RunbookBudgetRevisionRangeRule { min: 16, max: 32 }),
            Rc::new(RunbookBudgetKeyIdLengthRule { max_length: 255 }),
            Rc::new(RunbookBudgetRequiredTitleRule {
                message: "runbook budget needs this field".to_string(),
            }),
        ],
    }
}
