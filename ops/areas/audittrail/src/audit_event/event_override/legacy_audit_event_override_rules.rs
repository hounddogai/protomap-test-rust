use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy audit event override must pass. Check returns its problems, or none.
pub trait LegacyAuditEventOverrideRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAuditEventOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventOverrideNameLengthRule {
    pub max_length: i64,
}

impl LegacyAuditEventOverrideRule for LegacyAuditEventOverrideNameLengthRule {
    fn check(&self, item: &LegacyAuditEventOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventOverrideWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyAuditEventOverrideRule for LegacyAuditEventOverrideWeightRangeRule {
    fn check(&self, item: &LegacyAuditEventOverride) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventOverrideTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyAuditEventOverrideRule for LegacyAuditEventOverrideTtlSecondsRangeRule {
    fn check(&self, item: &LegacyAuditEventOverride) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventOverrideRequiredCodeRule {
    pub message: String,
}

impl LegacyAuditEventOverrideRule for LegacyAuditEventOverrideRequiredCodeRule {
    fn check(&self, item: &LegacyAuditEventOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventOverrideCodePrefixRule {
    pub prefix: String,
}

impl LegacyAuditEventOverrideRule for LegacyAuditEventOverrideCodePrefixRule {
    fn check(&self, item: &LegacyAuditEventOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventOverrideCountryLengthRule {
    pub max_length: i64,
}

impl LegacyAuditEventOverrideRule for LegacyAuditEventOverrideCountryLengthRule {
    fn check(&self, item: &LegacyAuditEventOverride) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy audit event override.
#[derive(Debug, Clone)]
pub struct LegacyAuditEventOverrideValidator {
    pub rules: Vec<Rc<dyn LegacyAuditEventOverrideRule>>,
}

impl LegacyAuditEventOverrideValidator {
    pub fn validate(&self, item: &LegacyAuditEventOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyAuditEventOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAuditEventOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_audit_event_override_validator() -> LegacyAuditEventOverrideValidator {
    LegacyAuditEventOverrideValidator {
        rules: vec![
            Rc::new(LegacyAuditEventOverrideNameLengthRule { max_length: 86400 })
                as Rc<dyn LegacyAuditEventOverrideRule>,
            Rc::new(LegacyAuditEventOverrideWeightRangeRule { min: 7.8, max: 1661.9 }),
            Rc::new(LegacyAuditEventOverrideTtlSecondsRangeRule { min: 255, max: 1000 }),
            Rc::new(LegacyAuditEventOverrideRequiredCodeRule {
                message: "legacy audit event override needs an entry".to_string(),
            }),
            Rc::new(LegacyAuditEventOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LegacyAuditEventOverrideCountryLengthRule { max_length: 80 }),
        ],
    }
}
