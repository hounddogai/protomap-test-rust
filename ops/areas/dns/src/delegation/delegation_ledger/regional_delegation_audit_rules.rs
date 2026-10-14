use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional delegation audit must pass. Check returns its problems, or none.
pub trait RegionalDelegationAuditRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDelegationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationAuditLabelLengthRule {
    pub max_length: i64,
}

impl RegionalDelegationAuditRule for RegionalDelegationAuditLabelLengthRule {
    fn check(&self, item: &RegionalDelegationAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationAuditCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDelegationAuditRule for RegionalDelegationAuditCreatedAtRangeRule {
    fn check(&self, item: &RegionalDelegationAudit) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationAuditCodePrefixRule {
    pub prefix: String,
}

impl RegionalDelegationAuditRule for RegionalDelegationAuditCodePrefixRule {
    fn check(&self, item: &RegionalDelegationAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationAuditRequiredHostRule {
    pub message: String,
}

impl RegionalDelegationAuditRule for RegionalDelegationAuditRequiredHostRule {
    fn check(&self, item: &RegionalDelegationAudit) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional delegation audit.
#[derive(Debug, Clone)]
pub struct RegionalDelegationAuditValidator {
    pub rules: Vec<Rc<dyn RegionalDelegationAuditRule>>,
}

impl RegionalDelegationAuditValidator {
    pub fn validate(&self, item: &RegionalDelegationAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalDelegationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDelegationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_delegation_audit_validator() -> RegionalDelegationAuditValidator {
    RegionalDelegationAuditValidator {
        rules: vec![
            Rc::new(RegionalDelegationAuditLabelLengthRule { max_length: 16 }) as Rc<dyn RegionalDelegationAuditRule>,
            Rc::new(RegionalDelegationAuditCreatedAtRangeRule { min: 32, max: 255 }),
            Rc::new(RegionalDelegationAuditCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionalDelegationAuditRequiredHostRule {
                message: "regional delegation audit needs a value".to_string(),
            }),
        ],
    }
}
