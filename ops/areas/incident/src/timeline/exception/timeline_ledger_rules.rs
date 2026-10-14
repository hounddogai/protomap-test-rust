use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline ledger must pass. Check returns its problems, or none.
pub trait TimelineLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TimelineLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineLedgerRequiredDescriptionRule {
    pub message: String,
}

impl TimelineLedgerRule for TimelineLedgerRequiredDescriptionRule {
    fn check(&self, item: &TimelineLedger) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineLedgerRequiredCodeRule {
    pub message: String,
}

impl TimelineLedgerRule for TimelineLedgerRequiredCodeRule {
    fn check(&self, item: &TimelineLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineLedgerCodePrefixRule {
    pub prefix: String,
}

impl TimelineLedgerRule for TimelineLedgerCodePrefixRule {
    fn check(&self, item: &TimelineLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline ledger.
#[derive(Debug, Clone)]
pub struct TimelineLedgerValidator {
    pub rules: Vec<Rc<dyn TimelineLedgerRule>>,
}

impl TimelineLedgerValidator {
    pub fn validate(&self, item: &TimelineLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_ledger_validator() -> TimelineLedgerValidator {
    TimelineLedgerValidator {
        rules: vec![
            Rc::new(TimelineLedgerRequiredDescriptionRule {
                message: "timeline ledger needs a value".to_string(),
            }) as Rc<dyn TimelineLedgerRule>,
            Rc::new(TimelineLedgerRequiredCodeRule {
                message: "timeline ledger needs a value".to_string(),
            }),
            Rc::new(TimelineLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
