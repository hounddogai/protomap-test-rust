use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential ledger must pass. Check returns its problems, or none.
pub trait CredentialLedgerRule: std::fmt::Debug {
    fn check(&self, item: &CredentialLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedgerCodePrefixRule {
    pub prefix: String,
}

impl CredentialLedgerRule for CredentialLedgerCodePrefixRule {
    fn check(&self, item: &CredentialLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedgerCodeLengthRule {
    pub max_length: i64,
}

impl CredentialLedgerRule for CredentialLedgerCodeLengthRule {
    fn check(&self, item: &CredentialLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedgerChecksumLengthRule {
    pub max_length: i64,
}

impl CredentialLedgerRule for CredentialLedgerChecksumLengthRule {
    fn check(&self, item: &CredentialLedger) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedgerCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialLedgerRule for CredentialLedgerCapacityRangeRule {
    fn check(&self, item: &CredentialLedger) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedgerLabelLengthRule {
    pub max_length: i64,
}

impl CredentialLedgerRule for CredentialLedgerLabelLengthRule {
    fn check(&self, item: &CredentialLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedgerSourceLengthRule {
    pub max_length: i64,
}

impl CredentialLedgerRule for CredentialLedgerSourceLengthRule {
    fn check(&self, item: &CredentialLedger) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential ledger.
#[derive(Debug, Clone)]
pub struct CredentialLedgerValidator {
    pub rules: Vec<Rc<dyn CredentialLedgerRule>>,
}

impl CredentialLedgerValidator {
    pub fn validate(&self, item: &CredentialLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_ledger_validator() -> CredentialLedgerValidator {
    CredentialLedgerValidator {
        rules: vec![
            Rc::new(CredentialLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn CredentialLedgerRule>,
            Rc::new(CredentialLedgerCodeLengthRule { max_length: 16 }),
            Rc::new(CredentialLedgerChecksumLengthRule { max_length: 1000 }),
            Rc::new(CredentialLedgerCapacityRangeRule { min: 255, max: 86400 }),
            Rc::new(CredentialLedgerLabelLengthRule { max_length: 64 }),
            Rc::new(CredentialLedgerSourceLengthRule { max_length: 3600 }),
        ],
    }
}
