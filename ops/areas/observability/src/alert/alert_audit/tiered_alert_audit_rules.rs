use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered alert audit must pass. Check returns its problems, or none.
pub trait TieredAlertAuditRule: std::fmt::Debug {
    fn check(&self, item: &TieredAlertAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAuditRequiredNameRule {
    pub message: String,
}

impl TieredAlertAuditRule for TieredAlertAuditRequiredNameRule {
    fn check(&self, item: &TieredAlertAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAuditScopeLengthRule {
    pub max_length: i64,
}

impl TieredAlertAuditRule for TieredAlertAuditScopeLengthRule {
    fn check(&self, item: &TieredAlertAudit) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAuditRequiredTimezoneRule {
    pub message: String,
}

impl TieredAlertAuditRule for TieredAlertAuditRequiredTimezoneRule {
    fn check(&self, item: &TieredAlertAudit) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAuditCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredAlertAuditRule for TieredAlertAuditCreatedAtRangeRule {
    fn check(&self, item: &TieredAlertAudit) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered alert audit.
#[derive(Debug, Clone)]
pub struct TieredAlertAuditValidator {
    pub rules: Vec<Rc<dyn TieredAlertAuditRule>>,
}

impl TieredAlertAuditValidator {
    pub fn validate(&self, item: &TieredAlertAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredAlertAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAlertAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_alert_audit_validator() -> TieredAlertAuditValidator {
    TieredAlertAuditValidator {
        rules: vec![
            Rc::new(TieredAlertAuditRequiredNameRule {
                message: "tiered alert audit needs a value".to_string(),
            }) as Rc<dyn TieredAlertAuditRule>,
            Rc::new(TieredAlertAuditScopeLengthRule { max_length: 255 }),
            Rc::new(TieredAlertAuditRequiredTimezoneRule {
                message: "tiered alert audit needs an entry".to_string(),
            }),
            Rc::new(TieredAlertAuditCreatedAtRangeRule { min: 1000, max: 1000 }),
        ],
    }
}
