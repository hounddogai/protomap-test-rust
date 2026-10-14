use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release batch must pass. Check returns its problems, or none.
pub trait ReleaseBatchRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ReleaseBatchRule for ReleaseBatchAllowedCodeRule {
    fn check(&self, item: &ReleaseBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBatchCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseBatchRule for ReleaseBatchCodeLengthRule {
    fn check(&self, item: &ReleaseBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBatchRequiredTitleRule {
    pub message: String,
}

impl ReleaseBatchRule for ReleaseBatchRequiredTitleRule {
    fn check(&self, item: &ReleaseBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBatchRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseBatchRule for ReleaseBatchRevisionRangeRule {
    fn check(&self, item: &ReleaseBatch) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBatchCodePrefixRule {
    pub prefix: String,
}

impl ReleaseBatchRule for ReleaseBatchCodePrefixRule {
    fn check(&self, item: &ReleaseBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBatchQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseBatchRule for ReleaseBatchQuotaUnitsRangeRule {
    fn check(&self, item: &ReleaseBatch) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a release batch.
#[derive(Debug, Clone)]
pub struct ReleaseBatchValidator {
    pub rules: Vec<Rc<dyn ReleaseBatchRule>>,
}

impl ReleaseBatchValidator {
    pub fn validate(&self, item: &ReleaseBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_batch_validator() -> ReleaseBatchValidator {
    ReleaseBatchValidator {
        rules: vec![
            Rc::new(ReleaseBatchAllowedCodeRule {
                allowed: vec!["default".to_string(), "capped".to_string(), "primary".to_string()],
            }) as Rc<dyn ReleaseBatchRule>,
            Rc::new(ReleaseBatchCodeLengthRule { max_length: 255 }),
            Rc::new(ReleaseBatchRequiredTitleRule {
                message: "release batch needs an entry".to_string(),
            }),
            Rc::new(ReleaseBatchRevisionRangeRule { min: 80, max: 3600 }),
            Rc::new(ReleaseBatchCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ReleaseBatchQuotaUnitsRangeRule { min: 1000, max: 3600 }),
        ],
    }
}
