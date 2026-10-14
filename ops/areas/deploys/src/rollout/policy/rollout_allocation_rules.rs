use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout allocation must pass. Check returns its problems, or none.
pub trait RolloutAllocationRule: std::fmt::Debug {
    fn check(&self, item: &RolloutAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocationCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutAllocationRule for RolloutAllocationCreatedAtRangeRule {
    fn check(&self, item: &RolloutAllocation) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocationNameLengthRule {
    pub max_length: i64,
}

impl RolloutAllocationRule for RolloutAllocationNameLengthRule {
    fn check(&self, item: &RolloutAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RolloutAllocationRule for RolloutAllocationAllowedCodeRule {
    fn check(&self, item: &RolloutAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocationRequiredChecksumRule {
    pub message: String,
}

impl RolloutAllocationRule for RolloutAllocationRequiredChecksumRule {
    fn check(&self, item: &RolloutAllocation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocationCodePrefixRule {
    pub prefix: String,
}

impl RolloutAllocationRule for RolloutAllocationCodePrefixRule {
    fn check(&self, item: &RolloutAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocationAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutAllocationRule for RolloutAllocationAttemptsRangeRule {
    fn check(&self, item: &RolloutAllocation) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollout allocation.
#[derive(Debug, Clone)]
pub struct RolloutAllocationValidator {
    pub rules: Vec<Rc<dyn RolloutAllocationRule>>,
}

impl RolloutAllocationValidator {
    pub fn validate(&self, item: &RolloutAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_allocation_validator() -> RolloutAllocationValidator {
    RolloutAllocationValidator {
        rules: vec![
            Rc::new(RolloutAllocationCreatedAtRangeRule { min: 80, max: 86400 }) as Rc<dyn RolloutAllocationRule>,
            Rc::new(RolloutAllocationNameLengthRule { max_length: 3600 }),
            Rc::new(RolloutAllocationAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "global".to_string(), "seasonal".to_string()],
            }),
            Rc::new(RolloutAllocationRequiredChecksumRule {
                message: "rollout allocation needs a value".to_string(),
            }),
            Rc::new(RolloutAllocationCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RolloutAllocationAttemptsRangeRule { min: 80, max: 80 }),
        ],
    }
}
