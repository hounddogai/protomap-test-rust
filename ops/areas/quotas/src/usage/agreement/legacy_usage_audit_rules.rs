use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy usage audit must pass. Check returns its problems, or none.
pub trait LegacyUsageAuditRule: std::fmt::Debug {
    fn check(&self, item: &LegacyUsageAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAuditCodeLengthRule {
    pub max_length: i64,
}

impl LegacyUsageAuditRule for LegacyUsageAuditCodeLengthRule {
    fn check(&self, item: &LegacyUsageAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAuditLabelLengthRule {
    pub max_length: i64,
}

impl LegacyUsageAuditRule for LegacyUsageAuditLabelLengthRule {
    fn check(&self, item: &LegacyUsageAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAuditCurrencyLengthRule {
    pub max_length: i64,
}

impl LegacyUsageAuditRule for LegacyUsageAuditCurrencyLengthRule {
    fn check(&self, item: &LegacyUsageAudit) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageAuditRequiredLabelRule {
    pub message: String,
}

impl LegacyUsageAuditRule for LegacyUsageAuditRequiredLabelRule {
    fn check(&self, item: &LegacyUsageAudit) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy usage audit.
#[derive(Debug, Clone)]
pub struct LegacyUsageAuditValidator {
    pub rules: Vec<Rc<dyn LegacyUsageAuditRule>>,
}

impl LegacyUsageAuditValidator {
    pub fn validate(&self, item: &LegacyUsageAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyUsageAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyUsageAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_usage_audit_validator() -> LegacyUsageAuditValidator {
    LegacyUsageAuditValidator {
        rules: vec![
            Rc::new(LegacyUsageAuditCodeLengthRule { max_length: 32 }) as Rc<dyn LegacyUsageAuditRule>,
            Rc::new(LegacyUsageAuditLabelLengthRule { max_length: 120 }),
            Rc::new(LegacyUsageAuditCurrencyLengthRule { max_length: 16 }),
            Rc::new(LegacyUsageAuditRequiredLabelRule {
                message: "legacy usage audit needs this field".to_string(),
            }),
        ],
    }
}
