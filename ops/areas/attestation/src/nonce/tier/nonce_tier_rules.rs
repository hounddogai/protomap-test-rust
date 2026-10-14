use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a nonce tier must pass. Check returns its problems, or none.
pub trait NonceTierRule: std::fmt::Debug {
    fn check(&self, item: &NonceTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTierTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceTierRule for NonceTierTtlSecondsRangeRule {
    fn check(&self, item: &NonceTier) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTierRequiredCodeRule {
    pub message: String,
}

impl NonceTierRule for NonceTierRequiredCodeRule {
    fn check(&self, item: &NonceTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTierLabelLengthRule {
    pub max_length: i64,
}

impl NonceTierRule for NonceTierLabelLengthRule {
    fn check(&self, item: &NonceTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTierCodeLengthRule {
    pub max_length: i64,
}

impl NonceTierRule for NonceTierCodeLengthRule {
    fn check(&self, item: &NonceTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTierCodePrefixRule {
    pub prefix: String,
}

impl NonceTierRule for NonceTierCodePrefixRule {
    fn check(&self, item: &NonceTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTierAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl NonceTierRule for NonceTierAllowedLabelRule {
    fn check(&self, item: &NonceTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce tier.
#[derive(Debug, Clone)]
pub struct NonceTierValidator {
    pub rules: Vec<Rc<dyn NonceTierRule>>,
}

impl NonceTierValidator {
    pub fn validate(&self, item: &NonceTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_tier_validator() -> NonceTierValidator {
    NonceTierValidator {
        rules: vec![
            Rc::new(NonceTierTtlSecondsRangeRule { min: 64, max: 3600 }) as Rc<dyn NonceTierRule>,
            Rc::new(NonceTierRequiredCodeRule {
                message: "nonce tier needs this field".to_string(),
            }),
            Rc::new(NonceTierLabelLengthRule { max_length: 32 }),
            Rc::new(NonceTierCodeLengthRule { max_length: 255 }),
            Rc::new(NonceTierCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(NonceTierAllowedLabelRule {
                allowed: vec!["capped".to_string(), "tiered".to_string(), "weighted".to_string()],
            }),
        ],
    }
}
