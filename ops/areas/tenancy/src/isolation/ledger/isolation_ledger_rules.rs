use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation ledger must pass. Check returns its problems, or none.
pub trait IsolationLedgerRule: std::fmt::Debug {
    fn check(&self, item: &IsolationLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationLedgerTitleLengthRule {
    pub max_length: i64,
}

impl IsolationLedgerRule for IsolationLedgerTitleLengthRule {
    fn check(&self, item: &IsolationLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationLedgerCodePrefixRule {
    pub prefix: String,
}

impl IsolationLedgerRule for IsolationLedgerCodePrefixRule {
    fn check(&self, item: &IsolationLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationLedgerNameLengthRule {
    pub max_length: i64,
}

impl IsolationLedgerRule for IsolationLedgerNameLengthRule {
    fn check(&self, item: &IsolationLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationLedgerDescriptionLengthRule {
    pub max_length: i64,
}

impl IsolationLedgerRule for IsolationLedgerDescriptionLengthRule {
    fn check(&self, item: &IsolationLedger) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationLedgerRequiredTitleRule {
    pub message: String,
}

impl IsolationLedgerRule for IsolationLedgerRequiredTitleRule {
    fn check(&self, item: &IsolationLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation ledger.
#[derive(Debug, Clone)]
pub struct IsolationLedgerValidator {
    pub rules: Vec<Rc<dyn IsolationLedgerRule>>,
}

impl IsolationLedgerValidator {
    pub fn validate(&self, item: &IsolationLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_ledger_validator() -> IsolationLedgerValidator {
    IsolationLedgerValidator {
        rules: vec![
            Rc::new(IsolationLedgerTitleLengthRule { max_length: 32 }) as Rc<dyn IsolationLedgerRule>,
            Rc::new(IsolationLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(IsolationLedgerNameLengthRule { max_length: 16 }),
            Rc::new(IsolationLedgerDescriptionLengthRule { max_length: 64 }),
            Rc::new(IsolationLedgerRequiredTitleRule {
                message: "isolation ledger needs this field".to_string(),
            }),
        ],
    }
}
