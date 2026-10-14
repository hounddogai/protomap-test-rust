use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary usage audit must pass. Check returns its problems, or none.
pub trait PrimaryUsageAuditRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryUsageAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAuditAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PrimaryUsageAuditRule for PrimaryUsageAuditAllowedNameRule {
    fn check(&self, item: &PrimaryUsageAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAuditNameLengthRule {
    pub max_length: i64,
}

impl PrimaryUsageAuditRule for PrimaryUsageAuditNameLengthRule {
    fn check(&self, item: &PrimaryUsageAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAuditCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryUsageAuditRule for PrimaryUsageAuditCodeLengthRule {
    fn check(&self, item: &PrimaryUsageAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAuditActiveGuardRule {
    pub limit: f64,
}

impl PrimaryUsageAuditRule for PrimaryUsageAuditActiveGuardRule {
    fn check(&self, item: &PrimaryUsageAudit) -> Vec<String> {
        if item.active && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while active", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAuditPrincipalLengthRule {
    pub max_length: i64,
}

impl PrimaryUsageAuditRule for PrimaryUsageAuditPrincipalLengthRule {
    fn check(&self, item: &PrimaryUsageAudit) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAuditSourceLengthRule {
    pub max_length: i64,
}

impl PrimaryUsageAuditRule for PrimaryUsageAuditSourceLengthRule {
    fn check(&self, item: &PrimaryUsageAudit) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary usage audit.
#[derive(Debug, Clone)]
pub struct PrimaryUsageAuditValidator {
    pub rules: Vec<Rc<dyn PrimaryUsageAuditRule>>,
}

impl PrimaryUsageAuditValidator {
    pub fn validate(&self, item: &PrimaryUsageAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryUsageAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryUsageAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_usage_audit_validator() -> PrimaryUsageAuditValidator {
    PrimaryUsageAuditValidator {
        rules: vec![
            Rc::new(PrimaryUsageAuditAllowedNameRule {
                allowed: vec!["tiered".to_string()],
            }) as Rc<dyn PrimaryUsageAuditRule>,
            Rc::new(PrimaryUsageAuditNameLengthRule { max_length: 3600 }),
            Rc::new(PrimaryUsageAuditCodeLengthRule { max_length: 86400 }),
            Rc::new(PrimaryUsageAuditActiveGuardRule { limit: 5.8 }),
            Rc::new(PrimaryUsageAuditPrincipalLengthRule { max_length: 255 }),
            Rc::new(PrimaryUsageAuditSourceLengthRule { max_length: 80 }),
        ],
    }
}
