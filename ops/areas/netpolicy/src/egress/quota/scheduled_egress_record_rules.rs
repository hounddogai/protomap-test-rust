use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled egress record must pass. Check returns its problems, or none.
pub trait ScheduledEgressRecordRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledEgressRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressRecordRequiredCountryRule {
    pub message: String,
}

impl ScheduledEgressRecordRule for ScheduledEgressRecordRequiredCountryRule {
    fn check(&self, item: &ScheduledEgressRecord) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressRecordCountryLengthRule {
    pub max_length: i64,
}

impl ScheduledEgressRecordRule for ScheduledEgressRecordCountryLengthRule {
    fn check(&self, item: &ScheduledEgressRecord) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressRecordCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledEgressRecordRule for ScheduledEgressRecordCodeLengthRule {
    fn check(&self, item: &ScheduledEgressRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled egress record.
#[derive(Debug, Clone)]
pub struct ScheduledEgressRecordValidator {
    pub rules: Vec<Rc<dyn ScheduledEgressRecordRule>>,
}

impl ScheduledEgressRecordValidator {
    pub fn validate(&self, item: &ScheduledEgressRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledEgressRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledEgressRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_egress_record_validator() -> ScheduledEgressRecordValidator {
    ScheduledEgressRecordValidator {
        rules: vec![
            Rc::new(ScheduledEgressRecordRequiredCountryRule {
                message: "scheduled egress record needs a value".to_string(),
            }) as Rc<dyn ScheduledEgressRecordRule>,
            Rc::new(ScheduledEgressRecordCountryLengthRule { max_length: 32 }),
            Rc::new(ScheduledEgressRecordCodeLengthRule { max_length: 1000 }),
        ],
    }
}
