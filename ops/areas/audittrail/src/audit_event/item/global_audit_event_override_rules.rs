use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global audit event override must pass. Check returns its problems, or none.
pub trait GlobalAuditEventOverrideRule: std::fmt::Debug {
    fn check(&self, item: &GlobalAuditEventOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventOverrideAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl GlobalAuditEventOverrideRule for GlobalAuditEventOverrideAllowedLabelRule {
    fn check(&self, item: &GlobalAuditEventOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventOverrideRequiredNameRule {
    pub message: String,
}

impl GlobalAuditEventOverrideRule for GlobalAuditEventOverrideRequiredNameRule {
    fn check(&self, item: &GlobalAuditEventOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventOverrideCodePrefixRule {
    pub prefix: String,
}

impl GlobalAuditEventOverrideRule for GlobalAuditEventOverrideCodePrefixRule {
    fn check(&self, item: &GlobalAuditEventOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventOverrideScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalAuditEventOverrideRule for GlobalAuditEventOverrideScoreRangeRule {
    fn check(&self, item: &GlobalAuditEventOverride) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventOverrideRequiredCodeRule {
    pub message: String,
}

impl GlobalAuditEventOverrideRule for GlobalAuditEventOverrideRequiredCodeRule {
    fn check(&self, item: &GlobalAuditEventOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global audit event override.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventOverrideValidator {
    pub rules: Vec<Rc<dyn GlobalAuditEventOverrideRule>>,
}

impl GlobalAuditEventOverrideValidator {
    pub fn validate(&self, item: &GlobalAuditEventOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalAuditEventOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalAuditEventOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_audit_event_override_validator() -> GlobalAuditEventOverrideValidator {
    GlobalAuditEventOverrideValidator {
        rules: vec![
            Rc::new(GlobalAuditEventOverrideAllowedLabelRule { allowed: Vec::new() })
                as Rc<dyn GlobalAuditEventOverrideRule>,
            Rc::new(GlobalAuditEventOverrideRequiredNameRule {
                message: "global audit event override needs a value".to_string(),
            }),
            Rc::new(GlobalAuditEventOverrideCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GlobalAuditEventOverrideScoreRangeRule { min: 9.7, max: 2854.4 }),
            Rc::new(GlobalAuditEventOverrideRequiredCodeRule {
                message: "global audit event override needs an entry".to_string(),
            }),
        ],
    }
}
