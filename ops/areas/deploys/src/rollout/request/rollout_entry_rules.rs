use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout entry must pass. Check returns its problems, or none.
pub trait RolloutEntryRule: std::fmt::Debug {
    fn check(&self, item: &RolloutEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEntryCodeLengthRule {
    pub max_length: i64,
}

impl RolloutEntryRule for RolloutEntryCodeLengthRule {
    fn check(&self, item: &RolloutEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEntryRequiredNameRule {
    pub message: String,
}

impl RolloutEntryRule for RolloutEntryRequiredNameRule {
    fn check(&self, item: &RolloutEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEntryRequiredCodeRule {
    pub message: String,
}

impl RolloutEntryRule for RolloutEntryRequiredCodeRule {
    fn check(&self, item: &RolloutEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEntryRequiredUnitRule {
    pub message: String,
}

impl RolloutEntryRule for RolloutEntryRequiredUnitRule {
    fn check(&self, item: &RolloutEntry) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEntryUnitLengthRule {
    pub max_length: i64,
}

impl RolloutEntryRule for RolloutEntryUnitLengthRule {
    fn check(&self, item: &RolloutEntry) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEntryTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutEntryRule for RolloutEntryTtlSecondsRangeRule {
    fn check(&self, item: &RolloutEntry) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollout entry.
#[derive(Debug, Clone)]
pub struct RolloutEntryValidator {
    pub rules: Vec<Rc<dyn RolloutEntryRule>>,
}

impl RolloutEntryValidator {
    pub fn validate(&self, item: &RolloutEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_entry_validator() -> RolloutEntryValidator {
    RolloutEntryValidator {
        rules: vec![
            Rc::new(RolloutEntryCodeLengthRule { max_length: 1000 }) as Rc<dyn RolloutEntryRule>,
            Rc::new(RolloutEntryRequiredNameRule {
                message: "rollout entry needs a value".to_string(),
            }),
            Rc::new(RolloutEntryRequiredCodeRule {
                message: "rollout entry needs an entry".to_string(),
            }),
            Rc::new(RolloutEntryRequiredUnitRule {
                message: "rollout entry needs an entry".to_string(),
            }),
            Rc::new(RolloutEntryUnitLengthRule { max_length: 86400 }),
            Rc::new(RolloutEntryTtlSecondsRangeRule { min: 255, max: 255 }),
        ],
    }
}
