use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled nonce score must pass. Check returns its problems, or none.
pub trait ScheduledNonceScoreRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledNonceScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceScoreCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledNonceScoreRule for ScheduledNonceScoreCreatedAtRangeRule {
    fn check(&self, item: &ScheduledNonceScore) -> Vec<String> {
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
pub struct ScheduledNonceScoreOwnerIdLengthRule {
    pub max_length: i64,
}

impl ScheduledNonceScoreRule for ScheduledNonceScoreOwnerIdLengthRule {
    fn check(&self, item: &ScheduledNonceScore) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceScoreCodePrefixRule {
    pub prefix: String,
}

impl ScheduledNonceScoreRule for ScheduledNonceScoreCodePrefixRule {
    fn check(&self, item: &ScheduledNonceScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceScoreAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ScheduledNonceScoreRule for ScheduledNonceScoreAllowedNameRule {
    fn check(&self, item: &ScheduledNonceScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceScoreCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledNonceScoreRule for ScheduledNonceScoreCodeLengthRule {
    fn check(&self, item: &ScheduledNonceScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceScoreRequiredCodeRule {
    pub message: String,
}

impl ScheduledNonceScoreRule for ScheduledNonceScoreRequiredCodeRule {
    fn check(&self, item: &ScheduledNonceScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled nonce score.
#[derive(Debug, Clone)]
pub struct ScheduledNonceScoreValidator {
    pub rules: Vec<Rc<dyn ScheduledNonceScoreRule>>,
}

impl ScheduledNonceScoreValidator {
    pub fn validate(&self, item: &ScheduledNonceScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledNonceScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledNonceScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_nonce_score_validator() -> ScheduledNonceScoreValidator {
    ScheduledNonceScoreValidator {
        rules: vec![
            Rc::new(ScheduledNonceScoreCreatedAtRangeRule { min: 64, max: 120 }) as Rc<dyn ScheduledNonceScoreRule>,
            Rc::new(ScheduledNonceScoreOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(ScheduledNonceScoreCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ScheduledNonceScoreAllowedNameRule { allowed: Vec::new() }),
            Rc::new(ScheduledNonceScoreCodeLengthRule { max_length: 80 }),
            Rc::new(ScheduledNonceScoreRequiredCodeRule {
                message: "scheduled nonce score needs an entry".to_string(),
            }),
        ],
    }
}
