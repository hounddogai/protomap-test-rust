use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered grant ledger must pass. Check returns its problems, or none.
pub trait TieredGrantLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TieredGrantLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantLedgerFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredGrantLedgerRule for TieredGrantLedgerFloorValueRangeRule {
    fn check(&self, item: &TieredGrantLedger) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantLedgerRequiredCodeRule {
    pub message: String,
}

impl TieredGrantLedgerRule for TieredGrantLedgerRequiredCodeRule {
    fn check(&self, item: &TieredGrantLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantLedgerIssuerLengthRule {
    pub max_length: i64,
}

impl TieredGrantLedgerRule for TieredGrantLedgerIssuerLengthRule {
    fn check(&self, item: &TieredGrantLedger) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered grant ledger.
#[derive(Debug, Clone)]
pub struct TieredGrantLedgerValidator {
    pub rules: Vec<Rc<dyn TieredGrantLedgerRule>>,
}

impl TieredGrantLedgerValidator {
    pub fn validate(&self, item: &TieredGrantLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredGrantLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredGrantLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_grant_ledger_validator() -> TieredGrantLedgerValidator {
    TieredGrantLedgerValidator {
        rules: vec![
            Rc::new(TieredGrantLedgerFloorValueRangeRule { min: 2.5, max: 3394.9 }) as Rc<dyn TieredGrantLedgerRule>,
            Rc::new(TieredGrantLedgerRequiredCodeRule {
                message: "tiered grant ledger needs an entry".to_string(),
            }),
            Rc::new(TieredGrantLedgerIssuerLengthRule { max_length: 16 }),
        ],
    }
}
