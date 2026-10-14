use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze entry must pass. Check returns its problems, or none.
pub trait FreezeEntryRule: std::fmt::Debug {
    fn check(&self, item: &FreezeEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeEntryRequiredContactEmailRule {
    pub message: String,
}

impl FreezeEntryRule for FreezeEntryRequiredContactEmailRule {
    fn check(&self, item: &FreezeEntry) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeEntryCodePrefixRule {
    pub prefix: String,
}

impl FreezeEntryRule for FreezeEntryCodePrefixRule {
    fn check(&self, item: &FreezeEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeEntryCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeEntryRule for FreezeEntryCreatedAtRangeRule {
    fn check(&self, item: &FreezeEntry) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeEntryRequiredUnitRule {
    pub message: String,
}

impl FreezeEntryRule for FreezeEntryRequiredUnitRule {
    fn check(&self, item: &FreezeEntry) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeEntryCodeLengthRule {
    pub max_length: i64,
}

impl FreezeEntryRule for FreezeEntryCodeLengthRule {
    fn check(&self, item: &FreezeEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze entry.
#[derive(Debug, Clone)]
pub struct FreezeEntryValidator {
    pub rules: Vec<Rc<dyn FreezeEntryRule>>,
}

impl FreezeEntryValidator {
    pub fn validate(&self, item: &FreezeEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_entry_validator() -> FreezeEntryValidator {
    FreezeEntryValidator {
        rules: vec![
            Rc::new(FreezeEntryRequiredContactEmailRule {
                message: "freeze entry needs a value".to_string(),
            }) as Rc<dyn FreezeEntryRule>,
            Rc::new(FreezeEntryCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(FreezeEntryCreatedAtRangeRule { min: 64, max: 86400 }),
            Rc::new(FreezeEntryRequiredUnitRule {
                message: "freeze entry needs an entry".to_string(),
            }),
            Rc::new(FreezeEntryCodeLengthRule { max_length: 255 }),
        ],
    }
}
