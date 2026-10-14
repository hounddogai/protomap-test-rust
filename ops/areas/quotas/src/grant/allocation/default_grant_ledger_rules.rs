use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default grant ledger must pass. Check returns its problems, or none.
pub trait DefaultGrantLedgerRule: std::fmt::Debug {
    fn check(&self, item: &DefaultGrantLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantLedgerRequiredLabelRule {
    pub message: String,
}

impl DefaultGrantLedgerRule for DefaultGrantLedgerRequiredLabelRule {
    fn check(&self, item: &DefaultGrantLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantLedgerRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultGrantLedgerRule for DefaultGrantLedgerRatioRangeRule {
    fn check(&self, item: &DefaultGrantLedger) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantLedgerNameLengthRule {
    pub max_length: i64,
}

impl DefaultGrantLedgerRule for DefaultGrantLedgerNameLengthRule {
    fn check(&self, item: &DefaultGrantLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantLedgerCodeLengthRule {
    pub max_length: i64,
}

impl DefaultGrantLedgerRule for DefaultGrantLedgerCodeLengthRule {
    fn check(&self, item: &DefaultGrantLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default grant ledger.
#[derive(Debug, Clone)]
pub struct DefaultGrantLedgerValidator {
    pub rules: Vec<Rc<dyn DefaultGrantLedgerRule>>,
}

impl DefaultGrantLedgerValidator {
    pub fn validate(&self, item: &DefaultGrantLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultGrantLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultGrantLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_grant_ledger_validator() -> DefaultGrantLedgerValidator {
    DefaultGrantLedgerValidator {
        rules: vec![
            Rc::new(DefaultGrantLedgerRequiredLabelRule {
                message: "default grant ledger needs a value".to_string(),
            }) as Rc<dyn DefaultGrantLedgerRule>,
            Rc::new(DefaultGrantLedgerRatioRangeRule { min: 0.3, max: 708.1 }),
            Rc::new(DefaultGrantLedgerNameLengthRule { max_length: 32 }),
            Rc::new(DefaultGrantLedgerCodeLengthRule { max_length: 16 }),
        ],
    }
}
