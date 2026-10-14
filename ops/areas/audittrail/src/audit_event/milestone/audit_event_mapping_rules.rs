use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event mapping must pass. Check returns its problems, or none.
pub trait AuditEventMappingRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMappingRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventMappingRule for AuditEventMappingRetriesRangeRule {
    fn check(&self, item: &AuditEventMapping) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMappingAudienceLengthRule {
    pub max_length: i64,
}

impl AuditEventMappingRule for AuditEventMappingAudienceLengthRule {
    fn check(&self, item: &AuditEventMapping) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMappingRequiredCodeRule {
    pub message: String,
}

impl AuditEventMappingRule for AuditEventMappingRequiredCodeRule {
    fn check(&self, item: &AuditEventMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMappingRequiredAudienceRule {
    pub message: String,
}

impl AuditEventMappingRule for AuditEventMappingRequiredAudienceRule {
    fn check(&self, item: &AuditEventMapping) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event mapping.
#[derive(Debug, Clone)]
pub struct AuditEventMappingValidator {
    pub rules: Vec<Rc<dyn AuditEventMappingRule>>,
}

impl AuditEventMappingValidator {
    pub fn validate(&self, item: &AuditEventMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_mapping_validator() -> AuditEventMappingValidator {
    AuditEventMappingValidator {
        rules: vec![
            Rc::new(AuditEventMappingRetriesRangeRule { min: 32, max: 80 }) as Rc<dyn AuditEventMappingRule>,
            Rc::new(AuditEventMappingAudienceLengthRule { max_length: 1000 }),
            Rc::new(AuditEventMappingRequiredCodeRule {
                message: "audit event mapping needs a value".to_string(),
            }),
            Rc::new(AuditEventMappingRequiredAudienceRule {
                message: "audit event mapping needs a value".to_string(),
            }),
        ],
    }
}
