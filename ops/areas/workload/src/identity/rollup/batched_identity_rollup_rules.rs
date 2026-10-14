use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched identity rollup must pass. Check returns its problems, or none.
pub trait BatchedIdentityRollupRule: std::fmt::Debug {
    fn check(&self, item: &BatchedIdentityRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollupRequiredTitleRule {
    pub message: String,
}

impl BatchedIdentityRollupRule for BatchedIdentityRollupRequiredTitleRule {
    fn check(&self, item: &BatchedIdentityRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollupCodeLengthRule {
    pub max_length: i64,
}

impl BatchedIdentityRollupRule for BatchedIdentityRollupCodeLengthRule {
    fn check(&self, item: &BatchedIdentityRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollupCodePrefixRule {
    pub prefix: String,
}

impl BatchedIdentityRollupRule for BatchedIdentityRollupCodePrefixRule {
    fn check(&self, item: &BatchedIdentityRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollupCipherLengthRule {
    pub max_length: i64,
}

impl BatchedIdentityRollupRule for BatchedIdentityRollupCipherLengthRule {
    fn check(&self, item: &BatchedIdentityRollup) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollupAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl BatchedIdentityRollupRule for BatchedIdentityRollupAllowedCipherRule {
    fn check(&self, item: &BatchedIdentityRollup) -> Vec<String> {
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
pub struct BatchedIdentityRollupRequiredCodeRule {
    pub message: String,
}

impl BatchedIdentityRollupRule for BatchedIdentityRollupRequiredCodeRule {
    fn check(&self, item: &BatchedIdentityRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched identity rollup.
#[derive(Debug, Clone)]
pub struct BatchedIdentityRollupValidator {
    pub rules: Vec<Rc<dyn BatchedIdentityRollupRule>>,
}

impl BatchedIdentityRollupValidator {
    pub fn validate(&self, item: &BatchedIdentityRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedIdentityRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedIdentityRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_identity_rollup_validator() -> BatchedIdentityRollupValidator {
    BatchedIdentityRollupValidator {
        rules: vec![
            Rc::new(BatchedIdentityRollupRequiredTitleRule {
                message: "batched identity rollup needs a value".to_string(),
            }) as Rc<dyn BatchedIdentityRollupRule>,
            Rc::new(BatchedIdentityRollupCodeLengthRule { max_length: 80 }),
            Rc::new(BatchedIdentityRollupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BatchedIdentityRollupCipherLengthRule { max_length: 120 }),
            Rc::new(BatchedIdentityRollupAllowedCipherRule { allowed: Vec::new() }),
            Rc::new(BatchedIdentityRollupRequiredCodeRule {
                message: "batched identity rollup needs a value".to_string(),
            }),
        ],
    }
}
