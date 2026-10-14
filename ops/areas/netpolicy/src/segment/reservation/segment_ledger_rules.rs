use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment ledger must pass. Check returns its problems, or none.
pub trait SegmentLedgerRule: std::fmt::Debug {
    fn check(&self, item: &SegmentLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLedgerRequiredNameRule {
    pub message: String,
}

impl SegmentLedgerRule for SegmentLedgerRequiredNameRule {
    fn check(&self, item: &SegmentLedger) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLedgerRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentLedgerRule for SegmentLedgerRetriesRangeRule {
    fn check(&self, item: &SegmentLedger) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLedgerRequiredCodeRule {
    pub message: String,
}

impl SegmentLedgerRule for SegmentLedgerRequiredCodeRule {
    fn check(&self, item: &SegmentLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLedgerCodeLengthRule {
    pub max_length: i64,
}

impl SegmentLedgerRule for SegmentLedgerCodeLengthRule {
    fn check(&self, item: &SegmentLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment ledger.
#[derive(Debug, Clone)]
pub struct SegmentLedgerValidator {
    pub rules: Vec<Rc<dyn SegmentLedgerRule>>,
}

impl SegmentLedgerValidator {
    pub fn validate(&self, item: &SegmentLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_ledger_validator() -> SegmentLedgerValidator {
    SegmentLedgerValidator {
        rules: vec![
            Rc::new(SegmentLedgerRequiredNameRule {
                message: "segment ledger needs a value".to_string(),
            }) as Rc<dyn SegmentLedgerRule>,
            Rc::new(SegmentLedgerRetriesRangeRule { min: 255, max: 255 }),
            Rc::new(SegmentLedgerRequiredCodeRule {
                message: "segment ledger needs an entry".to_string(),
            }),
            Rc::new(SegmentLedgerCodeLengthRule { max_length: 64 }),
        ],
    }
}
