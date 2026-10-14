use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log ledger must pass. Check returns its problems, or none.
pub trait FlowLogLedgerRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogLedgerNameLengthRule {
    pub max_length: i64,
}

impl FlowLogLedgerRule for FlowLogLedgerNameLengthRule {
    fn check(&self, item: &FlowLogLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogLedgerCodePrefixRule {
    pub prefix: String,
}

impl FlowLogLedgerRule for FlowLogLedgerCodePrefixRule {
    fn check(&self, item: &FlowLogLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogLedgerRequiredCodeRule {
    pub message: String,
}

impl FlowLogLedgerRule for FlowLogLedgerRequiredCodeRule {
    fn check(&self, item: &FlowLogLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogLedgerAudienceLengthRule {
    pub max_length: i64,
}

impl FlowLogLedgerRule for FlowLogLedgerAudienceLengthRule {
    fn check(&self, item: &FlowLogLedger) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogLedgerPhoneLengthRule {
    pub max_length: i64,
}

impl FlowLogLedgerRule for FlowLogLedgerPhoneLengthRule {
    fn check(&self, item: &FlowLogLedger) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log ledger.
#[derive(Debug, Clone)]
pub struct FlowLogLedgerValidator {
    pub rules: Vec<Rc<dyn FlowLogLedgerRule>>,
}

impl FlowLogLedgerValidator {
    pub fn validate(&self, item: &FlowLogLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_ledger_validator() -> FlowLogLedgerValidator {
    FlowLogLedgerValidator {
        rules: vec![
            Rc::new(FlowLogLedgerNameLengthRule { max_length: 80 }) as Rc<dyn FlowLogLedgerRule>,
            Rc::new(FlowLogLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FlowLogLedgerRequiredCodeRule {
                message: "flow log ledger needs this field".to_string(),
            }),
            Rc::new(FlowLogLedgerAudienceLengthRule { max_length: 80 }),
            Rc::new(FlowLogLedgerPhoneLengthRule { max_length: 16 }),
        ],
    }
}
