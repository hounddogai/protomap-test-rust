use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event threshold must pass. Check returns its problems, or none.
pub trait AuditEventThresholdRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventThresholdRequiredCodeRule {
    pub message: String,
}

impl AuditEventThresholdRule for AuditEventThresholdRequiredCodeRule {
    fn check(&self, item: &AuditEventThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventThresholdPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventThresholdRule for AuditEventThresholdPortRangeRule {
    fn check(&self, item: &AuditEventThreshold) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventThresholdCodePrefixRule {
    pub prefix: String,
}

impl AuditEventThresholdRule for AuditEventThresholdCodePrefixRule {
    fn check(&self, item: &AuditEventThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventThresholdRequiredUnitRule {
    pub message: String,
}

impl AuditEventThresholdRule for AuditEventThresholdRequiredUnitRule {
    fn check(&self, item: &AuditEventThreshold) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventThresholdAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl AuditEventThresholdRule for AuditEventThresholdAllowedUnitRule {
    fn check(&self, item: &AuditEventThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is inconsistent", item.unit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event threshold.
#[derive(Debug, Clone)]
pub struct AuditEventThresholdValidator {
    pub rules: Vec<Rc<dyn AuditEventThresholdRule>>,
}

impl AuditEventThresholdValidator {
    pub fn validate(&self, item: &AuditEventThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_threshold_validator() -> AuditEventThresholdValidator {
    AuditEventThresholdValidator {
        rules: vec![
            Rc::new(AuditEventThresholdRequiredCodeRule {
                message: "audit event threshold needs a value".to_string(),
            }) as Rc<dyn AuditEventThresholdRule>,
            Rc::new(AuditEventThresholdPortRangeRule { min: 32, max: 80 }),
            Rc::new(AuditEventThresholdCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(AuditEventThresholdRequiredUnitRule {
                message: "audit event threshold needs a value".to_string(),
            }),
            Rc::new(AuditEventThresholdAllowedUnitRule { allowed: Vec::new() }),
        ],
    }
}
