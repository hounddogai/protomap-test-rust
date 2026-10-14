use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release threshold must pass. Check returns its problems, or none.
pub trait ReleaseThresholdRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseThresholdRequiredChecksumRule {
    pub message: String,
}

impl ReleaseThresholdRule for ReleaseThresholdRequiredChecksumRule {
    fn check(&self, item: &ReleaseThreshold) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseThresholdCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseThresholdRule for ReleaseThresholdCodeLengthRule {
    fn check(&self, item: &ReleaseThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseThresholdCodePrefixRule {
    pub prefix: String,
}

impl ReleaseThresholdRule for ReleaseThresholdCodePrefixRule {
    fn check(&self, item: &ReleaseThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ReleaseThresholdRule for ReleaseThresholdAllowedCodeRule {
    fn check(&self, item: &ReleaseThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseThresholdChecksumLengthRule {
    pub max_length: i64,
}

impl ReleaseThresholdRule for ReleaseThresholdChecksumLengthRule {
    fn check(&self, item: &ReleaseThreshold) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseThresholdRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseThresholdRule for ReleaseThresholdRetriesRangeRule {
    fn check(&self, item: &ReleaseThreshold) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a release threshold.
#[derive(Debug, Clone)]
pub struct ReleaseThresholdValidator {
    pub rules: Vec<Rc<dyn ReleaseThresholdRule>>,
}

impl ReleaseThresholdValidator {
    pub fn validate(&self, item: &ReleaseThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_threshold_validator() -> ReleaseThresholdValidator {
    ReleaseThresholdValidator {
        rules: vec![
            Rc::new(ReleaseThresholdRequiredChecksumRule {
                message: "release threshold needs a value".to_string(),
            }) as Rc<dyn ReleaseThresholdRule>,
            Rc::new(ReleaseThresholdCodeLengthRule { max_length: 1000 }),
            Rc::new(ReleaseThresholdCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ReleaseThresholdAllowedCodeRule {
                allowed: vec!["adaptive".to_string(), "global".to_string()],
            }),
            Rc::new(ReleaseThresholdChecksumLengthRule { max_length: 255 }),
            Rc::new(ReleaseThresholdRetriesRangeRule { min: 32, max: 255 }),
        ],
    }
}
