use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress ledger must pass. Check returns its problems, or none.
pub trait EgressLedgerRule: std::fmt::Debug {
    fn check(&self, item: &EgressLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressLedgerRequiredOwnerIdRule {
    pub message: String,
}

impl EgressLedgerRule for EgressLedgerRequiredOwnerIdRule {
    fn check(&self, item: &EgressLedger) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressLedgerRequiredLabelRule {
    pub message: String,
}

impl EgressLedgerRule for EgressLedgerRequiredLabelRule {
    fn check(&self, item: &EgressLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressLedgerAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl EgressLedgerRule for EgressLedgerAllowedLabelRule {
    fn check(&self, item: &EgressLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressLedgerAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressLedgerRule for EgressLedgerAmountRangeRule {
    fn check(&self, item: &EgressLedger) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressLedgerLabelLengthRule {
    pub max_length: i64,
}

impl EgressLedgerRule for EgressLedgerLabelLengthRule {
    fn check(&self, item: &EgressLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress ledger.
#[derive(Debug, Clone)]
pub struct EgressLedgerValidator {
    pub rules: Vec<Rc<dyn EgressLedgerRule>>,
}

impl EgressLedgerValidator {
    pub fn validate(&self, item: &EgressLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_ledger_validator() -> EgressLedgerValidator {
    EgressLedgerValidator {
        rules: vec![
            Rc::new(EgressLedgerRequiredOwnerIdRule {
                message: "egress ledger needs this field".to_string(),
            }) as Rc<dyn EgressLedgerRule>,
            Rc::new(EgressLedgerRequiredLabelRule {
                message: "egress ledger needs a value".to_string(),
            }),
            Rc::new(EgressLedgerAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(EgressLedgerAmountRangeRule { min: 7.6, max: 2458.7 }),
            Rc::new(EgressLedgerLabelLengthRule { max_length: 64 }),
        ],
    }
}
