use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global runbook audit must pass. Check returns its problems, or none.
pub trait GlobalRunbookAuditRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRunbookAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAuditDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRunbookAuditRule for GlobalRunbookAuditDurationMinutesRangeRule {
    fn check(&self, item: &GlobalRunbookAudit) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAuditRequiredUnitRule {
    pub message: String,
}

impl GlobalRunbookAuditRule for GlobalRunbookAuditRequiredUnitRule {
    fn check(&self, item: &GlobalRunbookAudit) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAuditRequiredCodeRule {
    pub message: String,
}

impl GlobalRunbookAuditRule for GlobalRunbookAuditRequiredCodeRule {
    fn check(&self, item: &GlobalRunbookAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAuditLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRunbookAuditRule for GlobalRunbookAuditLimitRangeRule {
    fn check(&self, item: &GlobalRunbookAudit) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAuditUnitLengthRule {
    pub max_length: i64,
}

impl GlobalRunbookAuditRule for GlobalRunbookAuditUnitLengthRule {
    fn check(&self, item: &GlobalRunbookAudit) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAuditLabelLengthRule {
    pub max_length: i64,
}

impl GlobalRunbookAuditRule for GlobalRunbookAuditLabelLengthRule {
    fn check(&self, item: &GlobalRunbookAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global runbook audit.
#[derive(Debug, Clone)]
pub struct GlobalRunbookAuditValidator {
    pub rules: Vec<Rc<dyn GlobalRunbookAuditRule>>,
}

impl GlobalRunbookAuditValidator {
    pub fn validate(&self, item: &GlobalRunbookAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalRunbookAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRunbookAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_runbook_audit_validator() -> GlobalRunbookAuditValidator {
    GlobalRunbookAuditValidator {
        rules: vec![
            Rc::new(GlobalRunbookAuditDurationMinutesRangeRule { min: 32, max: 255 }) as Rc<dyn GlobalRunbookAuditRule>,
            Rc::new(GlobalRunbookAuditRequiredUnitRule {
                message: "global runbook audit needs an entry".to_string(),
            }),
            Rc::new(GlobalRunbookAuditRequiredCodeRule {
                message: "global runbook audit needs this field".to_string(),
            }),
            Rc::new(GlobalRunbookAuditLimitRangeRule { min: 16, max: 3600 }),
            Rc::new(GlobalRunbookAuditUnitLengthRule { max_length: 16 }),
            Rc::new(GlobalRunbookAuditLabelLengthRule { max_length: 1000 }),
        ],
    }
}
