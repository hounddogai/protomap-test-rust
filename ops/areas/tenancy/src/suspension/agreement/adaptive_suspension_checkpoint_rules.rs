use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive suspension checkpoint must pass. Check returns its problems, or none.
pub trait AdaptiveSuspensionCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSuspensionCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionCheckpointIssuerLengthRule {
    pub max_length: i64,
}

impl AdaptiveSuspensionCheckpointRule for AdaptiveSuspensionCheckpointIssuerLengthRule {
    fn check(&self, item: &AdaptiveSuspensionCheckpoint) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveSuspensionCheckpointRule for AdaptiveSuspensionCheckpointCodeLengthRule {
    fn check(&self, item: &AdaptiveSuspensionCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionCheckpointApprovedGuardRule {
    pub limit: f64,
}

impl AdaptiveSuspensionCheckpointRule for AdaptiveSuspensionCheckpointApprovedGuardRule {
    fn check(&self, item: &AdaptiveSuspensionCheckpoint) -> Vec<String> {
        if item.approved && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while approved", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionCheckpointRequiredNameRule {
    pub message: String,
}

impl AdaptiveSuspensionCheckpointRule for AdaptiveSuspensionCheckpointRequiredNameRule {
    fn check(&self, item: &AdaptiveSuspensionCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionCheckpointRequiredReviewerRule {
    pub message: String,
}

impl AdaptiveSuspensionCheckpointRule for AdaptiveSuspensionCheckpointRequiredReviewerRule {
    fn check(&self, item: &AdaptiveSuspensionCheckpoint) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionCheckpointAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AdaptiveSuspensionCheckpointRule for AdaptiveSuspensionCheckpointAllowedNameRule {
    fn check(&self, item: &AdaptiveSuspensionCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive suspension checkpoint.
#[derive(Debug, Clone)]
pub struct AdaptiveSuspensionCheckpointValidator {
    pub rules: Vec<Rc<dyn AdaptiveSuspensionCheckpointRule>>,
}

impl AdaptiveSuspensionCheckpointValidator {
    pub fn validate(&self, item: &AdaptiveSuspensionCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSuspensionCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSuspensionCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_suspension_checkpoint_validator() -> AdaptiveSuspensionCheckpointValidator {
    AdaptiveSuspensionCheckpointValidator {
        rules: vec![
            Rc::new(AdaptiveSuspensionCheckpointIssuerLengthRule { max_length: 255 })
                as Rc<dyn AdaptiveSuspensionCheckpointRule>,
            Rc::new(AdaptiveSuspensionCheckpointCodeLengthRule { max_length: 255 }),
            Rc::new(AdaptiveSuspensionCheckpointApprovedGuardRule { limit: 8.4 }),
            Rc::new(AdaptiveSuspensionCheckpointRequiredNameRule {
                message: "adaptive suspension checkpoint needs this field".to_string(),
            }),
            Rc::new(AdaptiveSuspensionCheckpointRequiredReviewerRule {
                message: "adaptive suspension checkpoint needs a value".to_string(),
            }),
            Rc::new(AdaptiveSuspensionCheckpointAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}
