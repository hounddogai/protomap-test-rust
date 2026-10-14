use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard ledger must pass. Check returns its problems, or none.
pub trait DashboardLedgerRule: std::fmt::Debug {
    fn check(&self, item: &DashboardLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardLedgerRequiredCodeRule {
    pub message: String,
}

impl DashboardLedgerRule for DashboardLedgerRequiredCodeRule {
    fn check(&self, item: &DashboardLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardLedgerCodePrefixRule {
    pub prefix: String,
}

impl DashboardLedgerRule for DashboardLedgerCodePrefixRule {
    fn check(&self, item: &DashboardLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardLedgerNameLengthRule {
    pub max_length: i64,
}

impl DashboardLedgerRule for DashboardLedgerNameLengthRule {
    fn check(&self, item: &DashboardLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardLedgerAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardLedgerRule for DashboardLedgerAmountRangeRule {
    fn check(&self, item: &DashboardLedger) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardLedgerCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardLedgerRule for DashboardLedgerCapacityRangeRule {
    fn check(&self, item: &DashboardLedger) -> Vec<String> {
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

/// Runs every rule of a dashboard ledger.
#[derive(Debug, Clone)]
pub struct DashboardLedgerValidator {
    pub rules: Vec<Rc<dyn DashboardLedgerRule>>,
}

impl DashboardLedgerValidator {
    pub fn validate(&self, item: &DashboardLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_ledger_validator() -> DashboardLedgerValidator {
    DashboardLedgerValidator {
        rules: vec![
            Rc::new(DashboardLedgerRequiredCodeRule {
                message: "dashboard ledger needs a value".to_string(),
            }) as Rc<dyn DashboardLedgerRule>,
            Rc::new(DashboardLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DashboardLedgerNameLengthRule { max_length: 1000 }),
            Rc::new(DashboardLedgerAmountRangeRule { min: 7.3, max: 4306.3 }),
            Rc::new(DashboardLedgerCapacityRangeRule { min: 64, max: 1000 }),
        ],
    }
}
