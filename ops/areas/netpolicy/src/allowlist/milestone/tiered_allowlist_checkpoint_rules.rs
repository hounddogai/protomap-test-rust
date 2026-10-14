use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered allowlist checkpoint must pass. Check returns its problems, or none.
pub trait TieredAllowlistCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &TieredAllowlistCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistCheckpointRequiredCodeRule {
    pub message: String,
}

impl TieredAllowlistCheckpointRule for TieredAllowlistCheckpointRequiredCodeRule {
    fn check(&self, item: &TieredAllowlistCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistCheckpointAllowedNameRule {
    pub allowed: Vec<String>,
}

impl TieredAllowlistCheckpointRule for TieredAllowlistCheckpointAllowedNameRule {
    fn check(&self, item: &TieredAllowlistCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistCheckpointActiveGuardRule {
    pub limit: i64,
}

impl TieredAllowlistCheckpointRule for TieredAllowlistCheckpointActiveGuardRule {
    fn check(&self, item: &TieredAllowlistCheckpoint) -> Vec<String> {
        if item.active && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while active",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistCheckpointNameLengthRule {
    pub max_length: i64,
}

impl TieredAllowlistCheckpointRule for TieredAllowlistCheckpointNameLengthRule {
    fn check(&self, item: &TieredAllowlistCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistCheckpointCodePrefixRule {
    pub prefix: String,
}

impl TieredAllowlistCheckpointRule for TieredAllowlistCheckpointCodePrefixRule {
    fn check(&self, item: &TieredAllowlistCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered allowlist checkpoint.
#[derive(Debug, Clone)]
pub struct TieredAllowlistCheckpointValidator {
    pub rules: Vec<Rc<dyn TieredAllowlistCheckpointRule>>,
}

impl TieredAllowlistCheckpointValidator {
    pub fn validate(&self, item: &TieredAllowlistCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredAllowlistCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAllowlistCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_allowlist_checkpoint_validator() -> TieredAllowlistCheckpointValidator {
    TieredAllowlistCheckpointValidator {
        rules: vec![
            Rc::new(TieredAllowlistCheckpointRequiredCodeRule {
                message: "tiered allowlist checkpoint needs this field".to_string(),
            }) as Rc<dyn TieredAllowlistCheckpointRule>,
            Rc::new(TieredAllowlistCheckpointAllowedNameRule {
                allowed: vec!["seasonal".to_string(), "capped".to_string(), "capped".to_string()],
            }),
            Rc::new(TieredAllowlistCheckpointActiveGuardRule { limit: 32 }),
            Rc::new(TieredAllowlistCheckpointNameLengthRule { max_length: 255 }),
            Rc::new(TieredAllowlistCheckpointCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
