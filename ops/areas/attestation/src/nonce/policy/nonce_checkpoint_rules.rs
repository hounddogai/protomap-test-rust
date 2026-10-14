use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce checkpoint must pass. Check returns its problems, or none.
pub trait NonceCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &NonceCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceCheckpointBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceCheckpointRule for NonceCheckpointBudgetRangeRule {
    fn check(&self, item: &NonceCheckpoint) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceCheckpointCodePrefixRule {
    pub prefix: String,
}

impl NonceCheckpointRule for NonceCheckpointCodePrefixRule {
    fn check(&self, item: &NonceCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceCheckpointNameLengthRule {
    pub max_length: i64,
}

impl NonceCheckpointRule for NonceCheckpointNameLengthRule {
    fn check(&self, item: &NonceCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceCheckpointRequiredNameRule {
    pub message: String,
}

impl NonceCheckpointRule for NonceCheckpointRequiredNameRule {
    fn check(&self, item: &NonceCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceCheckpointIssuerLengthRule {
    pub max_length: i64,
}

impl NonceCheckpointRule for NonceCheckpointIssuerLengthRule {
    fn check(&self, item: &NonceCheckpoint) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce checkpoint.
#[derive(Debug, Clone)]
pub struct NonceCheckpointValidator {
    pub rules: Vec<Rc<dyn NonceCheckpointRule>>,
}

impl NonceCheckpointValidator {
    pub fn validate(&self, item: &NonceCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_checkpoint_validator() -> NonceCheckpointValidator {
    NonceCheckpointValidator {
        rules: vec![
            Rc::new(NonceCheckpointBudgetRangeRule { min: 4.5, max: 159.4 }) as Rc<dyn NonceCheckpointRule>,
            Rc::new(NonceCheckpointCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(NonceCheckpointNameLengthRule { max_length: 255 }),
            Rc::new(NonceCheckpointRequiredNameRule {
                message: "nonce checkpoint needs a value".to_string(),
            }),
            Rc::new(NonceCheckpointIssuerLengthRule { max_length: 120 }),
        ],
    }
}
