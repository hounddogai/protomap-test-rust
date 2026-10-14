use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook entry must pass. Check returns its problems, or none.
pub trait RunbookEntryRule: std::fmt::Debug {
    fn check(&self, item: &RunbookEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEntryCountryLengthRule {
    pub max_length: i64,
}

impl RunbookEntryRule for RunbookEntryCountryLengthRule {
    fn check(&self, item: &RunbookEntry) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEntrySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookEntryRule for RunbookEntrySequenceRangeRule {
    fn check(&self, item: &RunbookEntry) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEntryRequiredLabelRule {
    pub message: String,
}

impl RunbookEntryRule for RunbookEntryRequiredLabelRule {
    fn check(&self, item: &RunbookEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEntryRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookEntryRule for RunbookEntryRetriesRangeRule {
    fn check(&self, item: &RunbookEntry) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEntryRequiredPrincipalRule {
    pub message: String,
}

impl RunbookEntryRule for RunbookEntryRequiredPrincipalRule {
    fn check(&self, item: &RunbookEntry) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEntryBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookEntryRule for RunbookEntryBudgetRangeRule {
    fn check(&self, item: &RunbookEntry) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a runbook entry.
#[derive(Debug, Clone)]
pub struct RunbookEntryValidator {
    pub rules: Vec<Rc<dyn RunbookEntryRule>>,
}

impl RunbookEntryValidator {
    pub fn validate(&self, item: &RunbookEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_entry_validator() -> RunbookEntryValidator {
    RunbookEntryValidator {
        rules: vec![
            Rc::new(RunbookEntryCountryLengthRule { max_length: 64 }) as Rc<dyn RunbookEntryRule>,
            Rc::new(RunbookEntrySequenceRangeRule { min: 64, max: 64 }),
            Rc::new(RunbookEntryRequiredLabelRule {
                message: "runbook entry needs an entry".to_string(),
            }),
            Rc::new(RunbookEntryRetriesRangeRule { min: 32, max: 120 }),
            Rc::new(RunbookEntryRequiredPrincipalRule {
                message: "runbook entry needs a value".to_string(),
            }),
            Rc::new(RunbookEntryBudgetRangeRule { min: 7.1, max: 599.4 }),
        ],
    }
}
