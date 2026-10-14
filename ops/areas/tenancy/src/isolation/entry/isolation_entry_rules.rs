use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation entry must pass. Check returns its problems, or none.
pub trait IsolationEntryRule: std::fmt::Debug {
    fn check(&self, item: &IsolationEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntryCodeLengthRule {
    pub max_length: i64,
}

impl IsolationEntryRule for IsolationEntryCodeLengthRule {
    fn check(&self, item: &IsolationEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntryLabelLengthRule {
    pub max_length: i64,
}

impl IsolationEntryRule for IsolationEntryLabelLengthRule {
    fn check(&self, item: &IsolationEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntryRequiredPrincipalRule {
    pub message: String,
}

impl IsolationEntryRule for IsolationEntryRequiredPrincipalRule {
    fn check(&self, item: &IsolationEntry) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntryPrincipalLengthRule {
    pub max_length: i64,
}

impl IsolationEntryRule for IsolationEntryPrincipalLengthRule {
    fn check(&self, item: &IsolationEntry) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntryRequiredSourceRule {
    pub message: String,
}

impl IsolationEntryRule for IsolationEntryRequiredSourceRule {
    fn check(&self, item: &IsolationEntry) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation entry.
#[derive(Debug, Clone)]
pub struct IsolationEntryValidator {
    pub rules: Vec<Rc<dyn IsolationEntryRule>>,
}

impl IsolationEntryValidator {
    pub fn validate(&self, item: &IsolationEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_entry_validator() -> IsolationEntryValidator {
    IsolationEntryValidator {
        rules: vec![
            Rc::new(IsolationEntryCodeLengthRule { max_length: 64 }) as Rc<dyn IsolationEntryRule>,
            Rc::new(IsolationEntryLabelLengthRule { max_length: 255 }),
            Rc::new(IsolationEntryRequiredPrincipalRule {
                message: "isolation entry needs an entry".to_string(),
            }),
            Rc::new(IsolationEntryPrincipalLengthRule { max_length: 86400 }),
            Rc::new(IsolationEntryRequiredSourceRule {
                message: "isolation entry needs this field".to_string(),
            }),
        ],
    }
}
