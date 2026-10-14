use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling isolation audit must pass. Check returns its problems, or none.
pub trait RollingIsolationAuditRule: std::fmt::Debug {
    fn check(&self, item: &RollingIsolationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationAuditRequiredLabelRule {
    pub message: String,
}

impl RollingIsolationAuditRule for RollingIsolationAuditRequiredLabelRule {
    fn check(&self, item: &RollingIsolationAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationAuditCodePrefixRule {
    pub prefix: String,
}

impl RollingIsolationAuditRule for RollingIsolationAuditCodePrefixRule {
    fn check(&self, item: &RollingIsolationAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationAuditPhoneLengthRule {
    pub max_length: i64,
}

impl RollingIsolationAuditRule for RollingIsolationAuditPhoneLengthRule {
    fn check(&self, item: &RollingIsolationAudit) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationAuditRequiredPhoneRule {
    pub message: String,
}

impl RollingIsolationAuditRule for RollingIsolationAuditRequiredPhoneRule {
    fn check(&self, item: &RollingIsolationAudit) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationAuditLabelLengthRule {
    pub max_length: i64,
}

impl RollingIsolationAuditRule for RollingIsolationAuditLabelLengthRule {
    fn check(&self, item: &RollingIsolationAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling isolation audit.
#[derive(Debug, Clone)]
pub struct RollingIsolationAuditValidator {
    pub rules: Vec<Rc<dyn RollingIsolationAuditRule>>,
}

impl RollingIsolationAuditValidator {
    pub fn validate(&self, item: &RollingIsolationAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingIsolationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingIsolationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_isolation_audit_validator() -> RollingIsolationAuditValidator {
    RollingIsolationAuditValidator {
        rules: vec![
            Rc::new(RollingIsolationAuditRequiredLabelRule {
                message: "rolling isolation audit needs this field".to_string(),
            }) as Rc<dyn RollingIsolationAuditRule>,
            Rc::new(RollingIsolationAuditCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RollingIsolationAuditPhoneLengthRule { max_length: 64 }),
            Rc::new(RollingIsolationAuditRequiredPhoneRule {
                message: "rolling isolation audit needs a value".to_string(),
            }),
            Rc::new(RollingIsolationAuditLabelLengthRule { max_length: 64 }),
        ],
    }
}
