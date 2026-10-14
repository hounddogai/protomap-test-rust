use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped policy audit must pass. Check returns its problems, or none.
pub trait CappedPolicyAuditRule: std::fmt::Debug {
    fn check(&self, item: &CappedPolicyAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAuditSourceLengthRule {
    pub max_length: i64,
}

impl CappedPolicyAuditRule for CappedPolicyAuditSourceLengthRule {
    fn check(&self, item: &CappedPolicyAudit) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAuditCodeLengthRule {
    pub max_length: i64,
}

impl CappedPolicyAuditRule for CappedPolicyAuditCodeLengthRule {
    fn check(&self, item: &CappedPolicyAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAuditLabelLengthRule {
    pub max_length: i64,
}

impl CappedPolicyAuditRule for CappedPolicyAuditLabelLengthRule {
    fn check(&self, item: &CappedPolicyAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAuditCodePrefixRule {
    pub prefix: String,
}

impl CappedPolicyAuditRule for CappedPolicyAuditCodePrefixRule {
    fn check(&self, item: &CappedPolicyAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAuditAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl CappedPolicyAuditRule for CappedPolicyAuditAllowedNotesRule {
    fn check(&self, item: &CappedPolicyAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is unknown", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAuditRequiredNotesRule {
    pub message: String,
}

impl CappedPolicyAuditRule for CappedPolicyAuditRequiredNotesRule {
    fn check(&self, item: &CappedPolicyAudit) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped policy audit.
#[derive(Debug, Clone)]
pub struct CappedPolicyAuditValidator {
    pub rules: Vec<Rc<dyn CappedPolicyAuditRule>>,
}

impl CappedPolicyAuditValidator {
    pub fn validate(&self, item: &CappedPolicyAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedPolicyAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPolicyAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_policy_audit_validator() -> CappedPolicyAuditValidator {
    CappedPolicyAuditValidator {
        rules: vec![
            Rc::new(CappedPolicyAuditSourceLengthRule { max_length: 32 }) as Rc<dyn CappedPolicyAuditRule>,
            Rc::new(CappedPolicyAuditCodeLengthRule { max_length: 255 }),
            Rc::new(CappedPolicyAuditLabelLengthRule { max_length: 80 }),
            Rc::new(CappedPolicyAuditCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedPolicyAuditAllowedNotesRule { allowed: Vec::new() }),
            Rc::new(CappedPolicyAuditRequiredNotesRule {
                message: "capped policy audit needs this field".to_string(),
            }),
        ],
    }
}
