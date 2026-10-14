use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy quote checkpoint must pass. Check returns its problems, or none.
pub trait LegacyQuoteCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &LegacyQuoteCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteCheckpointRequiredCodeRule {
    pub message: String,
}

impl LegacyQuoteCheckpointRule for LegacyQuoteCheckpointRequiredCodeRule {
    fn check(&self, item: &LegacyQuoteCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteCheckpointCodePrefixRule {
    pub prefix: String,
}

impl LegacyQuoteCheckpointRule for LegacyQuoteCheckpointCodePrefixRule {
    fn check(&self, item: &LegacyQuoteCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteCheckpointKeyIdLengthRule {
    pub max_length: i64,
}

impl LegacyQuoteCheckpointRule for LegacyQuoteCheckpointKeyIdLengthRule {
    fn check(&self, item: &LegacyQuoteCheckpoint) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy quote checkpoint.
#[derive(Debug, Clone)]
pub struct LegacyQuoteCheckpointValidator {
    pub rules: Vec<Rc<dyn LegacyQuoteCheckpointRule>>,
}

impl LegacyQuoteCheckpointValidator {
    pub fn validate(&self, item: &LegacyQuoteCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyQuoteCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyQuoteCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_quote_checkpoint_validator() -> LegacyQuoteCheckpointValidator {
    LegacyQuoteCheckpointValidator {
        rules: vec![
            Rc::new(LegacyQuoteCheckpointRequiredCodeRule {
                message: "legacy quote checkpoint needs a value".to_string(),
            }) as Rc<dyn LegacyQuoteCheckpointRule>,
            Rc::new(LegacyQuoteCheckpointCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LegacyQuoteCheckpointKeyIdLengthRule { max_length: 255 }),
        ],
    }
}
