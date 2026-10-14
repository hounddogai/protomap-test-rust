use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched probe window must pass. Check returns its problems, or none.
pub trait BatchedProbeWindowRule: std::fmt::Debug {
    fn check(&self, item: &BatchedProbeWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeWindowRequiredNameRule {
    pub message: String,
}

impl BatchedProbeWindowRule for BatchedProbeWindowRequiredNameRule {
    fn check(&self, item: &BatchedProbeWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeWindowPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedProbeWindowRule for BatchedProbeWindowPortRangeRule {
    fn check(&self, item: &BatchedProbeWindow) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeWindowCodePrefixRule {
    pub prefix: String,
}

impl BatchedProbeWindowRule for BatchedProbeWindowCodePrefixRule {
    fn check(&self, item: &BatchedProbeWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeWindowRequiredCodeRule {
    pub message: String,
}

impl BatchedProbeWindowRule for BatchedProbeWindowRequiredCodeRule {
    fn check(&self, item: &BatchedProbeWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched probe window.
#[derive(Debug, Clone)]
pub struct BatchedProbeWindowValidator {
    pub rules: Vec<Rc<dyn BatchedProbeWindowRule>>,
}

impl BatchedProbeWindowValidator {
    pub fn validate(&self, item: &BatchedProbeWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedProbeWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedProbeWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_probe_window_validator() -> BatchedProbeWindowValidator {
    BatchedProbeWindowValidator {
        rules: vec![
            Rc::new(BatchedProbeWindowRequiredNameRule {
                message: "batched probe window needs a value".to_string(),
            }) as Rc<dyn BatchedProbeWindowRule>,
            Rc::new(BatchedProbeWindowPortRangeRule { min: 16, max: 86400 }),
            Rc::new(BatchedProbeWindowCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(BatchedProbeWindowRequiredCodeRule {
                message: "batched probe window needs an entry".to_string(),
            }),
        ],
    }
}
