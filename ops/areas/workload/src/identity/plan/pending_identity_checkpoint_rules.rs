use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending identity checkpoint must pass. Check returns its problems, or none.
pub trait PendingIdentityCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &PendingIdentityCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityCheckpointCodePrefixRule {
    pub prefix: String,
}

impl PendingIdentityCheckpointRule for PendingIdentityCheckpointCodePrefixRule {
    fn check(&self, item: &PendingIdentityCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityCheckpointRequiredCountryRule {
    pub message: String,
}

impl PendingIdentityCheckpointRule for PendingIdentityCheckpointRequiredCountryRule {
    fn check(&self, item: &PendingIdentityCheckpoint) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityCheckpointNameLengthRule {
    pub max_length: i64,
}

impl PendingIdentityCheckpointRule for PendingIdentityCheckpointNameLengthRule {
    fn check(&self, item: &PendingIdentityCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityCheckpointQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingIdentityCheckpointRule for PendingIdentityCheckpointQuantityRangeRule {
    fn check(&self, item: &PendingIdentityCheckpoint) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityCheckpointRequiredNameRule {
    pub message: String,
}

impl PendingIdentityCheckpointRule for PendingIdentityCheckpointRequiredNameRule {
    fn check(&self, item: &PendingIdentityCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending identity checkpoint.
#[derive(Debug, Clone)]
pub struct PendingIdentityCheckpointValidator {
    pub rules: Vec<Rc<dyn PendingIdentityCheckpointRule>>,
}

impl PendingIdentityCheckpointValidator {
    pub fn validate(&self, item: &PendingIdentityCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingIdentityCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingIdentityCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_identity_checkpoint_validator() -> PendingIdentityCheckpointValidator {
    PendingIdentityCheckpointValidator {
        rules: vec![
            Rc::new(PendingIdentityCheckpointCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn PendingIdentityCheckpointRule>,
            Rc::new(PendingIdentityCheckpointRequiredCountryRule {
                message: "pending identity checkpoint needs a value".to_string(),
            }),
            Rc::new(PendingIdentityCheckpointNameLengthRule { max_length: 1000 }),
            Rc::new(PendingIdentityCheckpointQuantityRangeRule { min: 32, max: 255 }),
            Rc::new(PendingIdentityCheckpointRequiredNameRule {
                message: "pending identity checkpoint needs an entry".to_string(),
            }),
        ],
    }
}
