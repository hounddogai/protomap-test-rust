use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered verifier ledger must pass. Check returns its problems, or none.
pub trait TieredVerifierLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TieredVerifierLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierLedgerRequiredNamespaceRule {
    pub message: String,
}

impl TieredVerifierLedgerRule for TieredVerifierLedgerRequiredNamespaceRule {
    fn check(&self, item: &TieredVerifierLedger) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierLedgerNameLengthRule {
    pub max_length: i64,
}

impl TieredVerifierLedgerRule for TieredVerifierLedgerNameLengthRule {
    fn check(&self, item: &TieredVerifierLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierLedgerFingerprintLengthRule {
    pub max_length: i64,
}

impl TieredVerifierLedgerRule for TieredVerifierLedgerFingerprintLengthRule {
    fn check(&self, item: &TieredVerifierLedger) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierLedgerRequiredCodeRule {
    pub message: String,
}

impl TieredVerifierLedgerRule for TieredVerifierLedgerRequiredCodeRule {
    fn check(&self, item: &TieredVerifierLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierLedgerWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredVerifierLedgerRule for TieredVerifierLedgerWeightRangeRule {
    fn check(&self, item: &TieredVerifierLedger) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered verifier ledger.
#[derive(Debug, Clone)]
pub struct TieredVerifierLedgerValidator {
    pub rules: Vec<Rc<dyn TieredVerifierLedgerRule>>,
}

impl TieredVerifierLedgerValidator {
    pub fn validate(&self, item: &TieredVerifierLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredVerifierLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredVerifierLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_verifier_ledger_validator() -> TieredVerifierLedgerValidator {
    TieredVerifierLedgerValidator {
        rules: vec![
            Rc::new(TieredVerifierLedgerRequiredNamespaceRule {
                message: "tiered verifier ledger needs a value".to_string(),
            }) as Rc<dyn TieredVerifierLedgerRule>,
            Rc::new(TieredVerifierLedgerNameLengthRule { max_length: 32 }),
            Rc::new(TieredVerifierLedgerFingerprintLengthRule { max_length: 32 }),
            Rc::new(TieredVerifierLedgerRequiredCodeRule {
                message: "tiered verifier ledger needs this field".to_string(),
            }),
            Rc::new(TieredVerifierLedgerWeightRangeRule { min: 9.3, max: 2391.8 }),
        ],
    }
}
