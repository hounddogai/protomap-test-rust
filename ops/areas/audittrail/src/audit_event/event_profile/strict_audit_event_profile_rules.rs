use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict audit event profile must pass. Check returns its problems, or none.
pub trait StrictAuditEventProfileRule: std::fmt::Debug {
    fn check(&self, item: &StrictAuditEventProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventProfileCodePrefixRule {
    pub prefix: String,
}

impl StrictAuditEventProfileRule for StrictAuditEventProfileCodePrefixRule {
    fn check(&self, item: &StrictAuditEventProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventProfileRequiredCodeRule {
    pub message: String,
}

impl StrictAuditEventProfileRule for StrictAuditEventProfileRequiredCodeRule {
    fn check(&self, item: &StrictAuditEventProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventProfileRequiredDescriptionRule {
    pub message: String,
}

impl StrictAuditEventProfileRule for StrictAuditEventProfileRequiredDescriptionRule {
    fn check(&self, item: &StrictAuditEventProfile) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventProfileChannelLengthRule {
    pub max_length: i64,
}

impl StrictAuditEventProfileRule for StrictAuditEventProfileChannelLengthRule {
    fn check(&self, item: &StrictAuditEventProfile) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict audit event profile.
#[derive(Debug, Clone)]
pub struct StrictAuditEventProfileValidator {
    pub rules: Vec<Rc<dyn StrictAuditEventProfileRule>>,
}

impl StrictAuditEventProfileValidator {
    pub fn validate(&self, item: &StrictAuditEventProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictAuditEventProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAuditEventProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_audit_event_profile_validator() -> StrictAuditEventProfileValidator {
    StrictAuditEventProfileValidator {
        rules: vec![
            Rc::new(StrictAuditEventProfileCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn StrictAuditEventProfileRule>,
            Rc::new(StrictAuditEventProfileRequiredCodeRule {
                message: "strict audit event profile needs this field".to_string(),
            }),
            Rc::new(StrictAuditEventProfileRequiredDescriptionRule {
                message: "strict audit event profile needs a value".to_string(),
            }),
            Rc::new(StrictAuditEventProfileChannelLengthRule { max_length: 80 }),
        ],
    }
}
