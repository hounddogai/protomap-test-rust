use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary failover ledger must pass. Check returns its problems, or none.
pub trait PrimaryFailoverLedgerRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFailoverLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverLedgerUnitLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverLedgerRule for PrimaryFailoverLedgerUnitLengthRule {
    fn check(&self, item: &PrimaryFailoverLedger) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverLedgerAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryFailoverLedgerRule for PrimaryFailoverLedgerAmountRangeRule {
    fn check(&self, item: &PrimaryFailoverLedger) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverLedgerRequiredCodeRule {
    pub message: String,
}

impl PrimaryFailoverLedgerRule for PrimaryFailoverLedgerRequiredCodeRule {
    fn check(&self, item: &PrimaryFailoverLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverLedgerCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverLedgerRule for PrimaryFailoverLedgerCodeLengthRule {
    fn check(&self, item: &PrimaryFailoverLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverLedgerApprovedGuardRule {
    pub limit: i64,
}

impl PrimaryFailoverLedgerRule for PrimaryFailoverLedgerApprovedGuardRule {
    fn check(&self, item: &PrimaryFailoverLedger) -> Vec<String> {
        if item.approved && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while approved",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary failover ledger.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverLedgerValidator {
    pub rules: Vec<Rc<dyn PrimaryFailoverLedgerRule>>,
}

impl PrimaryFailoverLedgerValidator {
    pub fn validate(&self, item: &PrimaryFailoverLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryFailoverLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFailoverLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_failover_ledger_validator() -> PrimaryFailoverLedgerValidator {
    PrimaryFailoverLedgerValidator {
        rules: vec![
            Rc::new(PrimaryFailoverLedgerUnitLengthRule { max_length: 3600 }) as Rc<dyn PrimaryFailoverLedgerRule>,
            Rc::new(PrimaryFailoverLedgerAmountRangeRule { min: 1.9, max: 4540.9 }),
            Rc::new(PrimaryFailoverLedgerRequiredCodeRule {
                message: "primary failover ledger needs an entry".to_string(),
            }),
            Rc::new(PrimaryFailoverLedgerCodeLengthRule { max_length: 1000 }),
            Rc::new(PrimaryFailoverLedgerApprovedGuardRule { limit: 80 }),
        ],
    }
}
