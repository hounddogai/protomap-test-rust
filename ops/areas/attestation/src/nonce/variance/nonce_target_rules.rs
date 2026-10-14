use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce target must pass. Check returns its problems, or none.
pub trait NonceTargetRule: std::fmt::Debug {
    fn check(&self, item: &NonceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTargetNameLengthRule {
    pub max_length: i64,
}

impl NonceTargetRule for NonceTargetNameLengthRule {
    fn check(&self, item: &NonceTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTargetRequiredCodeRule {
    pub message: String,
}

impl NonceTargetRule for NonceTargetRequiredCodeRule {
    fn check(&self, item: &NonceTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTargetRequiredChannelRule {
    pub message: String,
}

impl NonceTargetRule for NonceTargetRequiredChannelRule {
    fn check(&self, item: &NonceTarget) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTargetCodePrefixRule {
    pub prefix: String,
}

impl NonceTargetRule for NonceTargetCodePrefixRule {
    fn check(&self, item: &NonceTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce target.
#[derive(Debug, Clone)]
pub struct NonceTargetValidator {
    pub rules: Vec<Rc<dyn NonceTargetRule>>,
}

impl NonceTargetValidator {
    pub fn validate(&self, item: &NonceTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_target_validator() -> NonceTargetValidator {
    NonceTargetValidator {
        rules: vec![
            Rc::new(NonceTargetNameLengthRule { max_length: 1000 }) as Rc<dyn NonceTargetRule>,
            Rc::new(NonceTargetRequiredCodeRule {
                message: "nonce target needs an entry".to_string(),
            }),
            Rc::new(NonceTargetRequiredChannelRule {
                message: "nonce target needs a value".to_string(),
            }),
            Rc::new(NonceTargetCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
