use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback ledger must pass. Check returns its problems, or none.
pub trait RollbackLedgerRule: std::fmt::Debug {
    fn check(&self, item: &RollbackLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackLedgerCodeLengthRule {
    pub max_length: i64,
}

impl RollbackLedgerRule for RollbackLedgerCodeLengthRule {
    fn check(&self, item: &RollbackLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackLedgerPrincipalLengthRule {
    pub max_length: i64,
}

impl RollbackLedgerRule for RollbackLedgerPrincipalLengthRule {
    fn check(&self, item: &RollbackLedger) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackLedgerRequiredTitleRule {
    pub message: String,
}

impl RollbackLedgerRule for RollbackLedgerRequiredTitleRule {
    fn check(&self, item: &RollbackLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback ledger.
#[derive(Debug, Clone)]
pub struct RollbackLedgerValidator {
    pub rules: Vec<Rc<dyn RollbackLedgerRule>>,
}

impl RollbackLedgerValidator {
    pub fn validate(&self, item: &RollbackLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_ledger_validator() -> RollbackLedgerValidator {
    RollbackLedgerValidator {
        rules: vec![
            Rc::new(RollbackLedgerCodeLengthRule { max_length: 16 }) as Rc<dyn RollbackLedgerRule>,
            Rc::new(RollbackLedgerPrincipalLengthRule { max_length: 255 }),
            Rc::new(RollbackLedgerRequiredTitleRule {
                message: "rollback ledger needs an entry".to_string(),
            }),
        ],
    }
}
