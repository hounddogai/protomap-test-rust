use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered health check ledger must pass. Check returns its problems, or none.
pub trait TieredHealthCheckLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TieredHealthCheckLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckLedgerRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredHealthCheckLedgerRule for TieredHealthCheckLedgerRevisionRangeRule {
    fn check(&self, item: &TieredHealthCheckLedger) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckLedgerRequiredCodeRule {
    pub message: String,
}

impl TieredHealthCheckLedgerRule for TieredHealthCheckLedgerRequiredCodeRule {
    fn check(&self, item: &TieredHealthCheckLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckLedgerRequiredCountryRule {
    pub message: String,
}

impl TieredHealthCheckLedgerRule for TieredHealthCheckLedgerRequiredCountryRule {
    fn check(&self, item: &TieredHealthCheckLedger) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckLedgerCodePrefixRule {
    pub prefix: String,
}

impl TieredHealthCheckLedgerRule for TieredHealthCheckLedgerCodePrefixRule {
    fn check(&self, item: &TieredHealthCheckLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckLedgerRequiredLabelRule {
    pub message: String,
}

impl TieredHealthCheckLedgerRule for TieredHealthCheckLedgerRequiredLabelRule {
    fn check(&self, item: &TieredHealthCheckLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckLedgerCodeLengthRule {
    pub max_length: i64,
}

impl TieredHealthCheckLedgerRule for TieredHealthCheckLedgerCodeLengthRule {
    fn check(&self, item: &TieredHealthCheckLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered health check ledger.
#[derive(Debug, Clone)]
pub struct TieredHealthCheckLedgerValidator {
    pub rules: Vec<Rc<dyn TieredHealthCheckLedgerRule>>,
}

impl TieredHealthCheckLedgerValidator {
    pub fn validate(&self, item: &TieredHealthCheckLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredHealthCheckLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredHealthCheckLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_health_check_ledger_validator() -> TieredHealthCheckLedgerValidator {
    TieredHealthCheckLedgerValidator {
        rules: vec![
            Rc::new(TieredHealthCheckLedgerRevisionRangeRule { min: 80, max: 255 })
                as Rc<dyn TieredHealthCheckLedgerRule>,
            Rc::new(TieredHealthCheckLedgerRequiredCodeRule {
                message: "tiered health check ledger needs this field".to_string(),
            }),
            Rc::new(TieredHealthCheckLedgerRequiredCountryRule {
                message: "tiered health check ledger needs a value".to_string(),
            }),
            Rc::new(TieredHealthCheckLedgerCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredHealthCheckLedgerRequiredLabelRule {
                message: "tiered health check ledger needs this field".to_string(),
            }),
            Rc::new(TieredHealthCheckLedgerCodeLengthRule { max_length: 16 }),
        ],
    }
}
