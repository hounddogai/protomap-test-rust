use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query ledger must pass. Check returns its problems, or none.
pub trait QueryLedgerRule: std::fmt::Debug {
    fn check(&self, item: &QueryLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedgerUnitLengthRule {
    pub max_length: i64,
}

impl QueryLedgerRule for QueryLedgerUnitLengthRule {
    fn check(&self, item: &QueryLedger) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedgerCodeLengthRule {
    pub max_length: i64,
}

impl QueryLedgerRule for QueryLedgerCodeLengthRule {
    fn check(&self, item: &QueryLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedgerEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryLedgerRule for QueryLedgerEndsAtRangeRule {
    fn check(&self, item: &QueryLedger) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedgerRequiredUnitRule {
    pub message: String,
}

impl QueryLedgerRule for QueryLedgerRequiredUnitRule {
    fn check(&self, item: &QueryLedger) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedgerNotesLengthRule {
    pub max_length: i64,
}

impl QueryLedgerRule for QueryLedgerNotesLengthRule {
    fn check(&self, item: &QueryLedger) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedgerRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryLedgerRule for QueryLedgerRetriesRangeRule {
    fn check(&self, item: &QueryLedger) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a query ledger.
#[derive(Debug, Clone)]
pub struct QueryLedgerValidator {
    pub rules: Vec<Rc<dyn QueryLedgerRule>>,
}

impl QueryLedgerValidator {
    pub fn validate(&self, item: &QueryLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_ledger_validator() -> QueryLedgerValidator {
    QueryLedgerValidator {
        rules: vec![
            Rc::new(QueryLedgerUnitLengthRule { max_length: 64 }) as Rc<dyn QueryLedgerRule>,
            Rc::new(QueryLedgerCodeLengthRule { max_length: 16 }),
            Rc::new(QueryLedgerEndsAtRangeRule { min: 86400, max: 86400 }),
            Rc::new(QueryLedgerRequiredUnitRule {
                message: "query ledger needs this field".to_string(),
            }),
            Rc::new(QueryLedgerNotesLengthRule { max_length: 255 }),
            Rc::new(QueryLedgerRetriesRangeRule { min: 3600, max: 3600 }),
        ],
    }
}
