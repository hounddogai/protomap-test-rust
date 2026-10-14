use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback egress override must pass. Check returns its problems, or none.
pub trait FallbackEgressOverrideRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEgressOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressOverrideCodeLengthRule {
    pub max_length: i64,
}

impl FallbackEgressOverrideRule for FallbackEgressOverrideCodeLengthRule {
    fn check(&self, item: &FallbackEgressOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressOverrideRequiredSourceRule {
    pub message: String,
}

impl FallbackEgressOverrideRule for FallbackEgressOverrideRequiredSourceRule {
    fn check(&self, item: &FallbackEgressOverride) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressOverrideSourceLengthRule {
    pub max_length: i64,
}

impl FallbackEgressOverrideRule for FallbackEgressOverrideSourceLengthRule {
    fn check(&self, item: &FallbackEgressOverride) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressOverrideRequiredCodeRule {
    pub message: String,
}

impl FallbackEgressOverrideRule for FallbackEgressOverrideRequiredCodeRule {
    fn check(&self, item: &FallbackEgressOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressOverrideRequiredNameRule {
    pub message: String,
}

impl FallbackEgressOverrideRule for FallbackEgressOverrideRequiredNameRule {
    fn check(&self, item: &FallbackEgressOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressOverrideCipherLengthRule {
    pub max_length: i64,
}

impl FallbackEgressOverrideRule for FallbackEgressOverrideCipherLengthRule {
    fn check(&self, item: &FallbackEgressOverride) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback egress override.
#[derive(Debug, Clone)]
pub struct FallbackEgressOverrideValidator {
    pub rules: Vec<Rc<dyn FallbackEgressOverrideRule>>,
}

impl FallbackEgressOverrideValidator {
    pub fn validate(&self, item: &FallbackEgressOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackEgressOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEgressOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_egress_override_validator() -> FallbackEgressOverrideValidator {
    FallbackEgressOverrideValidator {
        rules: vec![
            Rc::new(FallbackEgressOverrideCodeLengthRule { max_length: 3600 }) as Rc<dyn FallbackEgressOverrideRule>,
            Rc::new(FallbackEgressOverrideRequiredSourceRule {
                message: "fallback egress override needs an entry".to_string(),
            }),
            Rc::new(FallbackEgressOverrideSourceLengthRule { max_length: 255 }),
            Rc::new(FallbackEgressOverrideRequiredCodeRule {
                message: "fallback egress override needs a value".to_string(),
            }),
            Rc::new(FallbackEgressOverrideRequiredNameRule {
                message: "fallback egress override needs an entry".to_string(),
            }),
            Rc::new(FallbackEgressOverrideCipherLengthRule { max_length: 1000 }),
        ],
    }
}
