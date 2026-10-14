use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo ledger must pass. Check returns its problems, or none.
pub trait SloLedgerRule: std::fmt::Debug {
    fn check(&self, item: &SloLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloLedgerSourceLengthRule {
    pub max_length: i64,
}

impl SloLedgerRule for SloLedgerSourceLengthRule {
    fn check(&self, item: &SloLedger) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloLedgerLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloLedgerRule for SloLedgerLimitRangeRule {
    fn check(&self, item: &SloLedger) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloLedgerNameLengthRule {
    pub max_length: i64,
}

impl SloLedgerRule for SloLedgerNameLengthRule {
    fn check(&self, item: &SloLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloLedgerRequiredCodeRule {
    pub message: String,
}

impl SloLedgerRule for SloLedgerRequiredCodeRule {
    fn check(&self, item: &SloLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloLedgerRequiredSourceRule {
    pub message: String,
}

impl SloLedgerRule for SloLedgerRequiredSourceRule {
    fn check(&self, item: &SloLedger) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo ledger.
#[derive(Debug, Clone)]
pub struct SloLedgerValidator {
    pub rules: Vec<Rc<dyn SloLedgerRule>>,
}

impl SloLedgerValidator {
    pub fn validate(&self, item: &SloLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_ledger_validator() -> SloLedgerValidator {
    SloLedgerValidator {
        rules: vec![
            Rc::new(SloLedgerSourceLengthRule { max_length: 80 }) as Rc<dyn SloLedgerRule>,
            Rc::new(SloLedgerLimitRangeRule { min: 3600, max: 86400 }),
            Rc::new(SloLedgerNameLengthRule { max_length: 86400 }),
            Rc::new(SloLedgerRequiredCodeRule {
                message: "slo ledger needs this field".to_string(),
            }),
            Rc::new(SloLedgerRequiredSourceRule {
                message: "slo ledger needs an entry".to_string(),
            }),
        ],
    }
}
