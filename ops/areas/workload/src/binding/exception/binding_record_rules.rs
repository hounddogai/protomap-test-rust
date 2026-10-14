use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding record must pass. Check returns its problems, or none.
pub trait BindingRecordRule: std::fmt::Debug {
    fn check(&self, item: &BindingRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRecordCodePrefixRule {
    pub prefix: String,
}

impl BindingRecordRule for BindingRecordCodePrefixRule {
    fn check(&self, item: &BindingRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRecordAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BindingRecordRule for BindingRecordAllowedCodeRule {
    fn check(&self, item: &BindingRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRecordRequiredNameRule {
    pub message: String,
}

impl BindingRecordRule for BindingRecordRequiredNameRule {
    fn check(&self, item: &BindingRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRecordRequiredChecksumRule {
    pub message: String,
}

impl BindingRecordRule for BindingRecordRequiredChecksumRule {
    fn check(&self, item: &BindingRecord) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding record.
#[derive(Debug, Clone)]
pub struct BindingRecordValidator {
    pub rules: Vec<Rc<dyn BindingRecordRule>>,
}

impl BindingRecordValidator {
    pub fn validate(&self, item: &BindingRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_record_validator() -> BindingRecordValidator {
    BindingRecordValidator {
        rules: vec![
            Rc::new(BindingRecordCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn BindingRecordRule>,
            Rc::new(BindingRecordAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(BindingRecordRequiredNameRule {
                message: "binding record needs a value".to_string(),
            }),
            Rc::new(BindingRecordRequiredChecksumRule {
                message: "binding record needs an entry".to_string(),
            }),
        ],
    }
}
