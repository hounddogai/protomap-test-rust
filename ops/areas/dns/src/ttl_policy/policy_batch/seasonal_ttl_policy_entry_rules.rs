use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal ttl policy entry must pass. Check returns its problems, or none.
pub trait SeasonalTtlPolicyEntryRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTtlPolicyEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEntryChannelLengthRule {
    pub max_length: i64,
}

impl SeasonalTtlPolicyEntryRule for SeasonalTtlPolicyEntryChannelLengthRule {
    fn check(&self, item: &SeasonalTtlPolicyEntry) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEntryRequiredCodeRule {
    pub message: String,
}

impl SeasonalTtlPolicyEntryRule for SeasonalTtlPolicyEntryRequiredCodeRule {
    fn check(&self, item: &SeasonalTtlPolicyEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEntryBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTtlPolicyEntryRule for SeasonalTtlPolicyEntryBatchSizeRangeRule {
    fn check(&self, item: &SeasonalTtlPolicyEntry) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEntryRequiredChannelRule {
    pub message: String,
}

impl SeasonalTtlPolicyEntryRule for SeasonalTtlPolicyEntryRequiredChannelRule {
    fn check(&self, item: &SeasonalTtlPolicyEntry) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEntryAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl SeasonalTtlPolicyEntryRule for SeasonalTtlPolicyEntryAllowedTitleRule {
    fn check(&self, item: &SeasonalTtlPolicyEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal ttl policy entry.
#[derive(Debug, Clone)]
pub struct SeasonalTtlPolicyEntryValidator {
    pub rules: Vec<Rc<dyn SeasonalTtlPolicyEntryRule>>,
}

impl SeasonalTtlPolicyEntryValidator {
    pub fn validate(&self, item: &SeasonalTtlPolicyEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalTtlPolicyEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTtlPolicyEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_ttl_policy_entry_validator() -> SeasonalTtlPolicyEntryValidator {
    SeasonalTtlPolicyEntryValidator {
        rules: vec![
            Rc::new(SeasonalTtlPolicyEntryChannelLengthRule { max_length: 32 }) as Rc<dyn SeasonalTtlPolicyEntryRule>,
            Rc::new(SeasonalTtlPolicyEntryRequiredCodeRule {
                message: "seasonal ttl policy entry needs an entry".to_string(),
            }),
            Rc::new(SeasonalTtlPolicyEntryBatchSizeRangeRule { min: 32, max: 3600 }),
            Rc::new(SeasonalTtlPolicyEntryRequiredChannelRule {
                message: "seasonal ttl policy entry needs an entry".to_string(),
            }),
            Rc::new(SeasonalTtlPolicyEntryAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}
