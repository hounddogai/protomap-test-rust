use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary flow log record must pass. Check returns its problems, or none.
pub trait PrimaryFlowLogRecordRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFlowLogRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFlowLogRecordRequiredCodeRule {
    pub message: String,
}

impl PrimaryFlowLogRecordRule for PrimaryFlowLogRecordRequiredCodeRule {
    fn check(&self, item: &PrimaryFlowLogRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFlowLogRecordAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PrimaryFlowLogRecordRule for PrimaryFlowLogRecordAllowedLabelRule {
    fn check(&self, item: &PrimaryFlowLogRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFlowLogRecordExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryFlowLogRecordRule for PrimaryFlowLogRecordExpiresAtRangeRule {
    fn check(&self, item: &PrimaryFlowLogRecord) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFlowLogRecordRequiredKeyIdRule {
    pub message: String,
}

impl PrimaryFlowLogRecordRule for PrimaryFlowLogRecordRequiredKeyIdRule {
    fn check(&self, item: &PrimaryFlowLogRecord) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary flow log record.
#[derive(Debug, Clone)]
pub struct PrimaryFlowLogRecordValidator {
    pub rules: Vec<Rc<dyn PrimaryFlowLogRecordRule>>,
}

impl PrimaryFlowLogRecordValidator {
    pub fn validate(&self, item: &PrimaryFlowLogRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryFlowLogRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFlowLogRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_flow_log_record_validator() -> PrimaryFlowLogRecordValidator {
    PrimaryFlowLogRecordValidator {
        rules: vec![
            Rc::new(PrimaryFlowLogRecordRequiredCodeRule {
                message: "primary flow log record needs an entry".to_string(),
            }) as Rc<dyn PrimaryFlowLogRecordRule>,
            Rc::new(PrimaryFlowLogRecordAllowedLabelRule {
                allowed: vec!["adaptive".to_string(), "global".to_string(), "rolling".to_string()],
            }),
            Rc::new(PrimaryFlowLogRecordExpiresAtRangeRule { min: 64, max: 86400 }),
            Rc::new(PrimaryFlowLogRecordRequiredKeyIdRule {
                message: "primary flow log record needs this field".to_string(),
            }),
        ],
    }
}
