use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional usage audit must pass. Check returns its problems, or none.
pub trait RegionalUsageAuditRule: std::fmt::Debug {
    fn check(&self, item: &RegionalUsageAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageAuditCodeLengthRule {
    pub max_length: i64,
}

impl RegionalUsageAuditRule for RegionalUsageAuditCodeLengthRule {
    fn check(&self, item: &RegionalUsageAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageAuditRequiredCodeRule {
    pub message: String,
}

impl RegionalUsageAuditRule for RegionalUsageAuditRequiredCodeRule {
    fn check(&self, item: &RegionalUsageAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageAuditAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalUsageAuditRule for RegionalUsageAuditAllowedCodeRule {
    fn check(&self, item: &RegionalUsageAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageAuditCodePrefixRule {
    pub prefix: String,
}

impl RegionalUsageAuditRule for RegionalUsageAuditCodePrefixRule {
    fn check(&self, item: &RegionalUsageAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional usage audit.
#[derive(Debug, Clone)]
pub struct RegionalUsageAuditValidator {
    pub rules: Vec<Rc<dyn RegionalUsageAuditRule>>,
}

impl RegionalUsageAuditValidator {
    pub fn validate(&self, item: &RegionalUsageAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalUsageAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalUsageAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_usage_audit_validator() -> RegionalUsageAuditValidator {
    RegionalUsageAuditValidator {
        rules: vec![
            Rc::new(RegionalUsageAuditCodeLengthRule { max_length: 16 }) as Rc<dyn RegionalUsageAuditRule>,
            Rc::new(RegionalUsageAuditRequiredCodeRule {
                message: "regional usage audit needs an entry".to_string(),
            }),
            Rc::new(RegionalUsageAuditAllowedCodeRule {
                allowed: vec!["primary".to_string(), "regional".to_string()],
            }),
            Rc::new(RegionalUsageAuditCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
