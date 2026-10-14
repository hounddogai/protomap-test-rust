use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn ledger must pass. Check returns its problems, or none.
pub trait BudgetBurnLedgerRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnLedgerCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnLedgerRule for BudgetBurnLedgerCeilingRangeRule {
    fn check(&self, item: &BudgetBurnLedger) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnLedgerRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnLedgerRule for BudgetBurnLedgerRequiredCodeRule {
    fn check(&self, item: &BudgetBurnLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnLedgerNameLengthRule {
    pub max_length: i64,
}

impl BudgetBurnLedgerRule for BudgetBurnLedgerNameLengthRule {
    fn check(&self, item: &BudgetBurnLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnLedgerCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnLedgerRule for BudgetBurnLedgerCodePrefixRule {
    fn check(&self, item: &BudgetBurnLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnLedgerLocaleLengthRule {
    pub max_length: i64,
}

impl BudgetBurnLedgerRule for BudgetBurnLedgerLocaleLengthRule {
    fn check(&self, item: &BudgetBurnLedger) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn ledger.
#[derive(Debug, Clone)]
pub struct BudgetBurnLedgerValidator {
    pub rules: Vec<Rc<dyn BudgetBurnLedgerRule>>,
}

impl BudgetBurnLedgerValidator {
    pub fn validate(&self, item: &BudgetBurnLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_ledger_validator() -> BudgetBurnLedgerValidator {
    BudgetBurnLedgerValidator {
        rules: vec![
            Rc::new(BudgetBurnLedgerCeilingRangeRule { min: 2.9, max: 2403.8 }) as Rc<dyn BudgetBurnLedgerRule>,
            Rc::new(BudgetBurnLedgerRequiredCodeRule {
                message: "budget burn ledger needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnLedgerNameLengthRule { max_length: 86400 }),
            Rc::new(BudgetBurnLedgerCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BudgetBurnLedgerLocaleLengthRule { max_length: 120 }),
        ],
    }
}
