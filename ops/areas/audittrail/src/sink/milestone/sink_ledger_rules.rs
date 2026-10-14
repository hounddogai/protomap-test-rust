use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink ledger must pass. Check returns its problems, or none.
pub trait SinkLedgerRule: std::fmt::Debug {
    fn check(&self, item: &SinkLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkLedgerCodePrefixRule {
    pub prefix: String,
}

impl SinkLedgerRule for SinkLedgerCodePrefixRule {
    fn check(&self, item: &SinkLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkLedgerRequiredNameRule {
    pub message: String,
}

impl SinkLedgerRule for SinkLedgerRequiredNameRule {
    fn check(&self, item: &SinkLedger) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkLedgerNameLengthRule {
    pub max_length: i64,
}

impl SinkLedgerRule for SinkLedgerNameLengthRule {
    fn check(&self, item: &SinkLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink ledger.
#[derive(Debug, Clone)]
pub struct SinkLedgerValidator {
    pub rules: Vec<Rc<dyn SinkLedgerRule>>,
}

impl SinkLedgerValidator {
    pub fn validate(&self, item: &SinkLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_ledger_validator() -> SinkLedgerValidator {
    SinkLedgerValidator {
        rules: vec![
            Rc::new(SinkLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn SinkLedgerRule>,
            Rc::new(SinkLedgerRequiredNameRule {
                message: "sink ledger needs a value".to_string(),
            }),
            Rc::new(SinkLedgerNameLengthRule { max_length: 255 }),
        ],
    }
}
