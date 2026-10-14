use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual nonce checkpoint must pass. Check returns its problems, or none.
pub trait ManualNonceCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ManualNonceCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceCheckpointRequiredNameRule {
    pub message: String,
}

impl ManualNonceCheckpointRule for ManualNonceCheckpointRequiredNameRule {
    fn check(&self, item: &ManualNonceCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceCheckpointRequiredClusterRule {
    pub message: String,
}

impl ManualNonceCheckpointRule for ManualNonceCheckpointRequiredClusterRule {
    fn check(&self, item: &ManualNonceCheckpoint) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceCheckpointNameLengthRule {
    pub max_length: i64,
}

impl ManualNonceCheckpointRule for ManualNonceCheckpointNameLengthRule {
    fn check(&self, item: &ManualNonceCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceCheckpointCodePrefixRule {
    pub prefix: String,
}

impl ManualNonceCheckpointRule for ManualNonceCheckpointCodePrefixRule {
    fn check(&self, item: &ManualNonceCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual nonce checkpoint.
#[derive(Debug, Clone)]
pub struct ManualNonceCheckpointValidator {
    pub rules: Vec<Rc<dyn ManualNonceCheckpointRule>>,
}

impl ManualNonceCheckpointValidator {
    pub fn validate(&self, item: &ManualNonceCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualNonceCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualNonceCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_nonce_checkpoint_validator() -> ManualNonceCheckpointValidator {
    ManualNonceCheckpointValidator {
        rules: vec![
            Rc::new(ManualNonceCheckpointRequiredNameRule {
                message: "manual nonce checkpoint needs a value".to_string(),
            }) as Rc<dyn ManualNonceCheckpointRule>,
            Rc::new(ManualNonceCheckpointRequiredClusterRule {
                message: "manual nonce checkpoint needs an entry".to_string(),
            }),
            Rc::new(ManualNonceCheckpointNameLengthRule { max_length: 80 }),
            Rc::new(ManualNonceCheckpointCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
