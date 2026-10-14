use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage ledger must pass. Check returns its problems, or none.
pub trait OverageLedgerRule: std::fmt::Debug {
    fn check(&self, item: &OverageLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageLedgerCodeLengthRule {
    pub max_length: i64,
}

impl OverageLedgerRule for OverageLedgerCodeLengthRule {
    fn check(&self, item: &OverageLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageLedgerLabelLengthRule {
    pub max_length: i64,
}

impl OverageLedgerRule for OverageLedgerLabelLengthRule {
    fn check(&self, item: &OverageLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageLedgerVisibleGuardRule {
    pub limit: i64,
}

impl OverageLedgerRule for OverageLedgerVisibleGuardRule {
    fn check(&self, item: &OverageLedger) -> Vec<String> {
        if item.visible && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while visible",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageLedgerReviewerLengthRule {
    pub max_length: i64,
}

impl OverageLedgerRule for OverageLedgerReviewerLengthRule {
    fn check(&self, item: &OverageLedger) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageLedgerRequiredLabelRule {
    pub message: String,
}

impl OverageLedgerRule for OverageLedgerRequiredLabelRule {
    fn check(&self, item: &OverageLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageLedgerUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageLedgerRule for OverageLedgerUpdatedAtRangeRule {
    fn check(&self, item: &OverageLedger) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a overage ledger.
#[derive(Debug, Clone)]
pub struct OverageLedgerValidator {
    pub rules: Vec<Rc<dyn OverageLedgerRule>>,
}

impl OverageLedgerValidator {
    pub fn validate(&self, item: &OverageLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_ledger_validator() -> OverageLedgerValidator {
    OverageLedgerValidator {
        rules: vec![
            Rc::new(OverageLedgerCodeLengthRule { max_length: 1000 }) as Rc<dyn OverageLedgerRule>,
            Rc::new(OverageLedgerLabelLengthRule { max_length: 16 }),
            Rc::new(OverageLedgerVisibleGuardRule { limit: 80 }),
            Rc::new(OverageLedgerReviewerLengthRule { max_length: 3600 }),
            Rc::new(OverageLedgerRequiredLabelRule {
                message: "overage ledger needs an entry".to_string(),
            }),
            Rc::new(OverageLedgerUpdatedAtRangeRule { min: 80, max: 80 }),
        ],
    }
}
