use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event preference must pass. Check returns its problems, or none.
pub trait AuditEventPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPreferenceRequiredNameRule {
    pub message: String,
}

impl AuditEventPreferenceRule for AuditEventPreferenceRequiredNameRule {
    fn check(&self, item: &AuditEventPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPreferenceOwnerIdLengthRule {
    pub max_length: i64,
}

impl AuditEventPreferenceRule for AuditEventPreferenceOwnerIdLengthRule {
    fn check(&self, item: &AuditEventPreference) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPreferenceRequiredOwnerIdRule {
    pub message: String,
}

impl AuditEventPreferenceRule for AuditEventPreferenceRequiredOwnerIdRule {
    fn check(&self, item: &AuditEventPreference) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPreferenceQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventPreferenceRule for AuditEventPreferenceQuantityRangeRule {
    fn check(&self, item: &AuditEventPreference) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPreferenceRequiredClusterRule {
    pub message: String,
}

impl AuditEventPreferenceRule for AuditEventPreferenceRequiredClusterRule {
    fn check(&self, item: &AuditEventPreference) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPreferenceNameLengthRule {
    pub max_length: i64,
}

impl AuditEventPreferenceRule for AuditEventPreferenceNameLengthRule {
    fn check(&self, item: &AuditEventPreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event preference.
#[derive(Debug, Clone)]
pub struct AuditEventPreferenceValidator {
    pub rules: Vec<Rc<dyn AuditEventPreferenceRule>>,
}

impl AuditEventPreferenceValidator {
    pub fn validate(&self, item: &AuditEventPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_preference_validator() -> AuditEventPreferenceValidator {
    AuditEventPreferenceValidator {
        rules: vec![
            Rc::new(AuditEventPreferenceRequiredNameRule {
                message: "audit event preference needs an entry".to_string(),
            }) as Rc<dyn AuditEventPreferenceRule>,
            Rc::new(AuditEventPreferenceOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(AuditEventPreferenceRequiredOwnerIdRule {
                message: "audit event preference needs an entry".to_string(),
            }),
            Rc::new(AuditEventPreferenceQuantityRangeRule { min: 64, max: 3600 }),
            Rc::new(AuditEventPreferenceRequiredClusterRule {
                message: "audit event preference needs an entry".to_string(),
            }),
            Rc::new(AuditEventPreferenceNameLengthRule { max_length: 16 }),
        ],
    }
}
