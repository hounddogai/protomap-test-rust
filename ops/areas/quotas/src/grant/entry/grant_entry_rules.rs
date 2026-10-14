use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant entry must pass. Check returns its problems, or none.
pub trait GrantEntryRule: std::fmt::Debug {
    fn check(&self, item: &GrantEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEntryNameLengthRule {
    pub max_length: i64,
}

impl GrantEntryRule for GrantEntryNameLengthRule {
    fn check(&self, item: &GrantEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEntryRequiredTitleRule {
    pub message: String,
}

impl GrantEntryRule for GrantEntryRequiredTitleRule {
    fn check(&self, item: &GrantEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEntryRequiredNameRule {
    pub message: String,
}

impl GrantEntryRule for GrantEntryRequiredNameRule {
    fn check(&self, item: &GrantEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant entry.
#[derive(Debug, Clone)]
pub struct GrantEntryValidator {
    pub rules: Vec<Rc<dyn GrantEntryRule>>,
}

impl GrantEntryValidator {
    pub fn validate(&self, item: &GrantEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_entry_validator() -> GrantEntryValidator {
    GrantEntryValidator {
        rules: vec![
            Rc::new(GrantEntryNameLengthRule { max_length: 3600 }) as Rc<dyn GrantEntryRule>,
            Rc::new(GrantEntryRequiredTitleRule {
                message: "grant entry needs an entry".to_string(),
            }),
            Rc::new(GrantEntryRequiredNameRule {
                message: "grant entry needs an entry".to_string(),
            }),
        ],
    }
}
