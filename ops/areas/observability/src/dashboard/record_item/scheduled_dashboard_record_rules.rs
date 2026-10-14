use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled dashboard record must pass. Check returns its problems, or none.
pub trait ScheduledDashboardRecordRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledDashboardRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRecordRequiredLabelRule {
    pub message: String,
}

impl ScheduledDashboardRecordRule for ScheduledDashboardRecordRequiredLabelRule {
    fn check(&self, item: &ScheduledDashboardRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRecordRequiredPhoneRule {
    pub message: String,
}

impl ScheduledDashboardRecordRule for ScheduledDashboardRecordRequiredPhoneRule {
    fn check(&self, item: &ScheduledDashboardRecord) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRecordLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledDashboardRecordRule for ScheduledDashboardRecordLabelLengthRule {
    fn check(&self, item: &ScheduledDashboardRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRecordPhoneLengthRule {
    pub max_length: i64,
}

impl ScheduledDashboardRecordRule for ScheduledDashboardRecordPhoneLengthRule {
    fn check(&self, item: &ScheduledDashboardRecord) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRecordAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl ScheduledDashboardRecordRule for ScheduledDashboardRecordAllowedPhoneRule {
    fn check(&self, item: &ScheduledDashboardRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is expired", item.phone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled dashboard record.
#[derive(Debug, Clone)]
pub struct ScheduledDashboardRecordValidator {
    pub rules: Vec<Rc<dyn ScheduledDashboardRecordRule>>,
}

impl ScheduledDashboardRecordValidator {
    pub fn validate(&self, item: &ScheduledDashboardRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledDashboardRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledDashboardRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_dashboard_record_validator() -> ScheduledDashboardRecordValidator {
    ScheduledDashboardRecordValidator {
        rules: vec![
            Rc::new(ScheduledDashboardRecordRequiredLabelRule {
                message: "scheduled dashboard record needs an entry".to_string(),
            }) as Rc<dyn ScheduledDashboardRecordRule>,
            Rc::new(ScheduledDashboardRecordRequiredPhoneRule {
                message: "scheduled dashboard record needs an entry".to_string(),
            }),
            Rc::new(ScheduledDashboardRecordLabelLengthRule { max_length: 80 }),
            Rc::new(ScheduledDashboardRecordPhoneLengthRule { max_length: 80 }),
            Rc::new(ScheduledDashboardRecordAllowedPhoneRule {
                allowed: vec!["primary".to_string(), "regional".to_string()],
            }),
        ],
    }
}
