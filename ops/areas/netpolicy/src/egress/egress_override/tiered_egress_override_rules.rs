use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered egress override must pass. Check returns its problems, or none.
pub trait TieredEgressOverrideRule: std::fmt::Debug {
    fn check(&self, item: &TieredEgressOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressOverrideCipherLengthRule {
    pub max_length: i64,
}

impl TieredEgressOverrideRule for TieredEgressOverrideCipherLengthRule {
    fn check(&self, item: &TieredEgressOverride) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressOverrideCodePrefixRule {
    pub prefix: String,
}

impl TieredEgressOverrideRule for TieredEgressOverrideCodePrefixRule {
    fn check(&self, item: &TieredEgressOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressOverrideRequiredCipherRule {
    pub message: String,
}

impl TieredEgressOverrideRule for TieredEgressOverrideRequiredCipherRule {
    fn check(&self, item: &TieredEgressOverride) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressOverrideAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl TieredEgressOverrideRule for TieredEgressOverrideAllowedCipherRule {
    fn check(&self, item: &TieredEgressOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is below minimum", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressOverrideRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredEgressOverrideRule for TieredEgressOverrideRevisionRangeRule {
    fn check(&self, item: &TieredEgressOverride) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered egress override.
#[derive(Debug, Clone)]
pub struct TieredEgressOverrideValidator {
    pub rules: Vec<Rc<dyn TieredEgressOverrideRule>>,
}

impl TieredEgressOverrideValidator {
    pub fn validate(&self, item: &TieredEgressOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredEgressOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredEgressOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_egress_override_validator() -> TieredEgressOverrideValidator {
    TieredEgressOverrideValidator {
        rules: vec![
            Rc::new(TieredEgressOverrideCipherLengthRule { max_length: 1000 }) as Rc<dyn TieredEgressOverrideRule>,
            Rc::new(TieredEgressOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TieredEgressOverrideRequiredCipherRule {
                message: "tiered egress override needs a value".to_string(),
            }),
            Rc::new(TieredEgressOverrideAllowedCipherRule { allowed: Vec::new() }),
            Rc::new(TieredEgressOverrideRevisionRangeRule { min: 120, max: 3600 }),
        ],
    }
}
