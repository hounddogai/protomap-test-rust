use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered audit event override must pass. Check returns its problems, or none.
pub trait TieredAuditEventOverrideRule: std::fmt::Debug {
    fn check(&self, item: &TieredAuditEventOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventOverrideRequiredCodeRule {
    pub message: String,
}

impl TieredAuditEventOverrideRule for TieredAuditEventOverrideRequiredCodeRule {
    fn check(&self, item: &TieredAuditEventOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventOverrideCodePrefixRule {
    pub prefix: String,
}

impl TieredAuditEventOverrideRule for TieredAuditEventOverrideCodePrefixRule {
    fn check(&self, item: &TieredAuditEventOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventOverrideCodeLengthRule {
    pub max_length: i64,
}

impl TieredAuditEventOverrideRule for TieredAuditEventOverrideCodeLengthRule {
    fn check(&self, item: &TieredAuditEventOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventOverrideRequiredLabelRule {
    pub message: String,
}

impl TieredAuditEventOverrideRule for TieredAuditEventOverrideRequiredLabelRule {
    fn check(&self, item: &TieredAuditEventOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventOverrideDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredAuditEventOverrideRule for TieredAuditEventOverrideDurationMinutesRangeRule {
    fn check(&self, item: &TieredAuditEventOverride) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TieredAuditEventOverrideRule for TieredAuditEventOverrideAllowedCodeRule {
    fn check(&self, item: &TieredAuditEventOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered audit event override.
#[derive(Debug, Clone)]
pub struct TieredAuditEventOverrideValidator {
    pub rules: Vec<Rc<dyn TieredAuditEventOverrideRule>>,
}

impl TieredAuditEventOverrideValidator {
    pub fn validate(&self, item: &TieredAuditEventOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredAuditEventOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAuditEventOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_audit_event_override_validator() -> TieredAuditEventOverrideValidator {
    TieredAuditEventOverrideValidator {
        rules: vec![
            Rc::new(TieredAuditEventOverrideRequiredCodeRule {
                message: "tiered audit event override needs this field".to_string(),
            }) as Rc<dyn TieredAuditEventOverrideRule>,
            Rc::new(TieredAuditEventOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TieredAuditEventOverrideCodeLengthRule { max_length: 16 }),
            Rc::new(TieredAuditEventOverrideRequiredLabelRule {
                message: "tiered audit event override needs an entry".to_string(),
            }),
            Rc::new(TieredAuditEventOverrideDurationMinutesRangeRule { min: 64, max: 1000 }),
            Rc::new(TieredAuditEventOverrideAllowedCodeRule {
                allowed: vec!["manual".to_string(), "weighted".to_string()],
            }),
        ],
    }
}
