use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential audit must pass. Check returns its problems, or none.
pub trait CredentialAuditRule: std::fmt::Debug {
    fn check(&self, item: &CredentialAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAuditCategoryLengthRule {
    pub max_length: i64,
}

impl CredentialAuditRule for CredentialAuditCategoryLengthRule {
    fn check(&self, item: &CredentialAudit) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAuditMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialAuditRule for CredentialAuditMarginRangeRule {
    fn check(&self, item: &CredentialAudit) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAuditRequiredCodeRule {
    pub message: String,
}

impl CredentialAuditRule for CredentialAuditRequiredCodeRule {
    fn check(&self, item: &CredentialAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAuditRequiredLocaleRule {
    pub message: String,
}

impl CredentialAuditRule for CredentialAuditRequiredLocaleRule {
    fn check(&self, item: &CredentialAudit) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential audit.
#[derive(Debug, Clone)]
pub struct CredentialAuditValidator {
    pub rules: Vec<Rc<dyn CredentialAuditRule>>,
}

impl CredentialAuditValidator {
    pub fn validate(&self, item: &CredentialAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_audit_validator() -> CredentialAuditValidator {
    CredentialAuditValidator {
        rules: vec![
            Rc::new(CredentialAuditCategoryLengthRule { max_length: 3600 }) as Rc<dyn CredentialAuditRule>,
            Rc::new(CredentialAuditMarginRangeRule { min: 3.3, max: 2752.0 }),
            Rc::new(CredentialAuditRequiredCodeRule {
                message: "credential audit needs a value".to_string(),
            }),
            Rc::new(CredentialAuditRequiredLocaleRule {
                message: "credential audit needs an entry".to_string(),
            }),
        ],
    }
}
