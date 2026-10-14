use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy runbook audit must pass. Check returns its problems, or none.
pub trait LegacyRunbookAuditRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRunbookAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAuditHostLengthRule {
    pub max_length: i64,
}

impl LegacyRunbookAuditRule for LegacyRunbookAuditHostLengthRule {
    fn check(&self, item: &LegacyRunbookAudit) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAuditNameLengthRule {
    pub max_length: i64,
}

impl LegacyRunbookAuditRule for LegacyRunbookAuditNameLengthRule {
    fn check(&self, item: &LegacyRunbookAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAuditGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyRunbookAuditRule for LegacyRunbookAuditGraceDaysRangeRule {
    fn check(&self, item: &LegacyRunbookAudit) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAuditAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl LegacyRunbookAuditRule for LegacyRunbookAuditAllowedChecksumRule {
    fn check(&self, item: &LegacyRunbookAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is below minimum", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAuditArchivedGuardRule {
    pub limit: f64,
}

impl LegacyRunbookAuditRule for LegacyRunbookAuditArchivedGuardRule {
    fn check(&self, item: &LegacyRunbookAudit) -> Vec<String> {
        if item.archived && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while archived", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAuditRequiredCodeRule {
    pub message: String,
}

impl LegacyRunbookAuditRule for LegacyRunbookAuditRequiredCodeRule {
    fn check(&self, item: &LegacyRunbookAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy runbook audit.
#[derive(Debug, Clone)]
pub struct LegacyRunbookAuditValidator {
    pub rules: Vec<Rc<dyn LegacyRunbookAuditRule>>,
}

impl LegacyRunbookAuditValidator {
    pub fn validate(&self, item: &LegacyRunbookAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRunbookAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRunbookAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_runbook_audit_validator() -> LegacyRunbookAuditValidator {
    LegacyRunbookAuditValidator {
        rules: vec![
            Rc::new(LegacyRunbookAuditHostLengthRule { max_length: 1000 }) as Rc<dyn LegacyRunbookAuditRule>,
            Rc::new(LegacyRunbookAuditNameLengthRule { max_length: 1000 }),
            Rc::new(LegacyRunbookAuditGraceDaysRangeRule { min: 120, max: 120 }),
            Rc::new(LegacyRunbookAuditAllowedChecksumRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(LegacyRunbookAuditArchivedGuardRule { limit: 1.4 }),
            Rc::new(LegacyRunbookAuditRequiredCodeRule {
                message: "legacy runbook audit needs this field".to_string(),
            }),
        ],
    }
}
