use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped federation ledger must pass. Check returns its problems, or none.
pub trait CappedFederationLedgerRule: std::fmt::Debug {
    fn check(&self, item: &CappedFederationLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationLedgerPrincipalLengthRule {
    pub max_length: i64,
}

impl CappedFederationLedgerRule for CappedFederationLedgerPrincipalLengthRule {
    fn check(&self, item: &CappedFederationLedger) -> Vec<String> {
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
pub struct CappedFederationLedgerOwnerIdLengthRule {
    pub max_length: i64,
}

impl CappedFederationLedgerRule for CappedFederationLedgerOwnerIdLengthRule {
    fn check(&self, item: &CappedFederationLedger) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationLedgerTitleLengthRule {
    pub max_length: i64,
}

impl CappedFederationLedgerRule for CappedFederationLedgerTitleLengthRule {
    fn check(&self, item: &CappedFederationLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped federation ledger.
#[derive(Debug, Clone)]
pub struct CappedFederationLedgerValidator {
    pub rules: Vec<Rc<dyn CappedFederationLedgerRule>>,
}

impl CappedFederationLedgerValidator {
    pub fn validate(&self, item: &CappedFederationLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedFederationLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedFederationLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_federation_ledger_validator() -> CappedFederationLedgerValidator {
    CappedFederationLedgerValidator {
        rules: vec![
            Rc::new(CappedFederationLedgerPrincipalLengthRule { max_length: 86400 })
                as Rc<dyn CappedFederationLedgerRule>,
            Rc::new(CappedFederationLedgerOwnerIdLengthRule { max_length: 64 }),
            Rc::new(CappedFederationLedgerTitleLengthRule { max_length: 1000 }),
        ],
    }
}
