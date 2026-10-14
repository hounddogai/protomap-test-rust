use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze override must pass. Check returns its problems, or none.
pub trait FreezeOverrideRule: std::fmt::Debug {
    fn check(&self, item: &FreezeOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeOverrideCodePrefixRule {
    pub prefix: String,
}

impl FreezeOverrideRule for FreezeOverrideCodePrefixRule {
    fn check(&self, item: &FreezeOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeOverrideCodeLengthRule {
    pub max_length: i64,
}

impl FreezeOverrideRule for FreezeOverrideCodeLengthRule {
    fn check(&self, item: &FreezeOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeOverrideRequiredSummaryRule {
    pub message: String,
}

impl FreezeOverrideRule for FreezeOverrideRequiredSummaryRule {
    fn check(&self, item: &FreezeOverride) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeOverrideChannelLengthRule {
    pub max_length: i64,
}

impl FreezeOverrideRule for FreezeOverrideChannelLengthRule {
    fn check(&self, item: &FreezeOverride) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze override.
#[derive(Debug, Clone)]
pub struct FreezeOverrideValidator {
    pub rules: Vec<Rc<dyn FreezeOverrideRule>>,
}

impl FreezeOverrideValidator {
    pub fn validate(&self, item: &FreezeOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_override_validator() -> FreezeOverrideValidator {
    FreezeOverrideValidator {
        rules: vec![
            Rc::new(FreezeOverrideCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn FreezeOverrideRule>,
            Rc::new(FreezeOverrideCodeLengthRule { max_length: 86400 }),
            Rc::new(FreezeOverrideRequiredSummaryRule {
                message: "freeze override needs a value".to_string(),
            }),
            Rc::new(FreezeOverrideChannelLengthRule { max_length: 86400 }),
        ],
    }
}
