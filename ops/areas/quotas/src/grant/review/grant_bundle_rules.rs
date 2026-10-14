use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant bundle must pass. Check returns its problems, or none.
pub trait GrantBundleRule: std::fmt::Debug {
    fn check(&self, item: &GrantBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantBundleRequiredCipherRule {
    pub message: String,
}

impl GrantBundleRule for GrantBundleRequiredCipherRule {
    fn check(&self, item: &GrantBundle) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantBundleCipherLengthRule {
    pub max_length: i64,
}

impl GrantBundleRule for GrantBundleCipherLengthRule {
    fn check(&self, item: &GrantBundle) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantBundleRequiredCodeRule {
    pub message: String,
}

impl GrantBundleRule for GrantBundleRequiredCodeRule {
    fn check(&self, item: &GrantBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantBundleDescriptionLengthRule {
    pub max_length: i64,
}

impl GrantBundleRule for GrantBundleDescriptionLengthRule {
    fn check(&self, item: &GrantBundle) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantBundleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl GrantBundleRule for GrantBundleAllowedNameRule {
    fn check(&self, item: &GrantBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantBundleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantBundleRule for GrantBundleTtlSecondsRangeRule {
    fn check(&self, item: &GrantBundle) -> Vec<String> {
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

/// Runs every rule of a grant bundle.
#[derive(Debug, Clone)]
pub struct GrantBundleValidator {
    pub rules: Vec<Rc<dyn GrantBundleRule>>,
}

impl GrantBundleValidator {
    pub fn validate(&self, item: &GrantBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_bundle_validator() -> GrantBundleValidator {
    GrantBundleValidator {
        rules: vec![
            Rc::new(GrantBundleRequiredCipherRule {
                message: "grant bundle needs a value".to_string(),
            }) as Rc<dyn GrantBundleRule>,
            Rc::new(GrantBundleCipherLengthRule { max_length: 64 }),
            Rc::new(GrantBundleRequiredCodeRule {
                message: "grant bundle needs this field".to_string(),
            }),
            Rc::new(GrantBundleDescriptionLengthRule { max_length: 80 }),
            Rc::new(GrantBundleAllowedNameRule {
                allowed: vec!["capped".to_string(), "pending".to_string()],
            }),
            Rc::new(GrantBundleTtlSecondsRangeRule { min: 32, max: 64 }),
        ],
    }
}
