use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential adjustment must pass. Check returns its problems, or none.
pub trait CredentialAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &CredentialAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustmentRequiredLabelRule {
    pub message: String,
}

impl CredentialAdjustmentRule for CredentialAdjustmentRequiredLabelRule {
    fn check(&self, item: &CredentialAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl CredentialAdjustmentRule for CredentialAdjustmentCodeLengthRule {
    fn check(&self, item: &CredentialAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustmentRequiredChannelRule {
    pub message: String,
}

impl CredentialAdjustmentRule for CredentialAdjustmentRequiredChannelRule {
    fn check(&self, item: &CredentialAdjustment) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustmentEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialAdjustmentRule for CredentialAdjustmentEndsAtRangeRule {
    fn check(&self, item: &CredentialAdjustment) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustmentAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl CredentialAdjustmentRule for CredentialAdjustmentAllowedChannelRule {
    fn check(&self, item: &CredentialAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is expired", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustmentRequiredCodeRule {
    pub message: String,
}

impl CredentialAdjustmentRule for CredentialAdjustmentRequiredCodeRule {
    fn check(&self, item: &CredentialAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential adjustment.
#[derive(Debug, Clone)]
pub struct CredentialAdjustmentValidator {
    pub rules: Vec<Rc<dyn CredentialAdjustmentRule>>,
}

impl CredentialAdjustmentValidator {
    pub fn validate(&self, item: &CredentialAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_adjustment_validator() -> CredentialAdjustmentValidator {
    CredentialAdjustmentValidator {
        rules: vec![
            Rc::new(CredentialAdjustmentRequiredLabelRule {
                message: "credential adjustment needs a value".to_string(),
            }) as Rc<dyn CredentialAdjustmentRule>,
            Rc::new(CredentialAdjustmentCodeLengthRule { max_length: 64 }),
            Rc::new(CredentialAdjustmentRequiredChannelRule {
                message: "credential adjustment needs an entry".to_string(),
            }),
            Rc::new(CredentialAdjustmentEndsAtRangeRule { min: 3600, max: 3600 }),
            Rc::new(CredentialAdjustmentAllowedChannelRule { allowed: Vec::new() }),
            Rc::new(CredentialAdjustmentRequiredCodeRule {
                message: "credential adjustment needs an entry".to_string(),
            }),
        ],
    }
}
