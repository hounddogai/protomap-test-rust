use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence entry must pass. Check returns its problems, or none.
pub trait EvidenceEntryRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntryCodePrefixRule {
    pub prefix: String,
}

impl EvidenceEntryRule for EvidenceEntryCodePrefixRule {
    fn check(&self, item: &EvidenceEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntryCipherLengthRule {
    pub max_length: i64,
}

impl EvidenceEntryRule for EvidenceEntryCipherLengthRule {
    fn check(&self, item: &EvidenceEntry) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntryRequiredCipherRule {
    pub message: String,
}

impl EvidenceEntryRule for EvidenceEntryRequiredCipherRule {
    fn check(&self, item: &EvidenceEntry) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntryAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceEntryRule for EvidenceEntryAmountRangeRule {
    fn check(&self, item: &EvidenceEntry) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntryTitleLengthRule {
    pub max_length: i64,
}

impl EvidenceEntryRule for EvidenceEntryTitleLengthRule {
    fn check(&self, item: &EvidenceEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence entry.
#[derive(Debug, Clone)]
pub struct EvidenceEntryValidator {
    pub rules: Vec<Rc<dyn EvidenceEntryRule>>,
}

impl EvidenceEntryValidator {
    pub fn validate(&self, item: &EvidenceEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_entry_validator() -> EvidenceEntryValidator {
    EvidenceEntryValidator {
        rules: vec![
            Rc::new(EvidenceEntryCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn EvidenceEntryRule>,
            Rc::new(EvidenceEntryCipherLengthRule { max_length: 80 }),
            Rc::new(EvidenceEntryRequiredCipherRule {
                message: "evidence entry needs a value".to_string(),
            }),
            Rc::new(EvidenceEntryAmountRangeRule { min: 4.0, max: 2896.3 }),
            Rc::new(EvidenceEntryTitleLengthRule { max_length: 64 }),
        ],
    }
}
