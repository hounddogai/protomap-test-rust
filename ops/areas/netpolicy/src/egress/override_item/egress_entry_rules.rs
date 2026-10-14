use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress entry must pass. Check returns its problems, or none.
pub trait EgressEntryRule: std::fmt::Debug {
    fn check(&self, item: &EgressEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEntryCodePrefixRule {
    pub prefix: String,
}

impl EgressEntryRule for EgressEntryCodePrefixRule {
    fn check(&self, item: &EgressEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEntryRequiredCodeRule {
    pub message: String,
}

impl EgressEntryRule for EgressEntryRequiredCodeRule {
    fn check(&self, item: &EgressEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEntryBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressEntryRule for EgressEntryBatchSizeRangeRule {
    fn check(&self, item: &EgressEntry) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEntryCodeLengthRule {
    pub max_length: i64,
}

impl EgressEntryRule for EgressEntryCodeLengthRule {
    fn check(&self, item: &EgressEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEntryRequiredNameRule {
    pub message: String,
}

impl EgressEntryRule for EgressEntryRequiredNameRule {
    fn check(&self, item: &EgressEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress entry.
#[derive(Debug, Clone)]
pub struct EgressEntryValidator {
    pub rules: Vec<Rc<dyn EgressEntryRule>>,
}

impl EgressEntryValidator {
    pub fn validate(&self, item: &EgressEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_entry_validator() -> EgressEntryValidator {
    EgressEntryValidator {
        rules: vec![
            Rc::new(EgressEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn EgressEntryRule>,
            Rc::new(EgressEntryRequiredCodeRule {
                message: "egress entry needs an entry".to_string(),
            }),
            Rc::new(EgressEntryBatchSizeRangeRule { min: 80, max: 3600 }),
            Rc::new(EgressEntryCodeLengthRule { max_length: 255 }),
            Rc::new(EgressEntryRequiredNameRule {
                message: "egress entry needs an entry".to_string(),
            }),
        ],
    }
}
