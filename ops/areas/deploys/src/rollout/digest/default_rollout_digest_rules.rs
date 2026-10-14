use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default rollout digest must pass. Check returns its problems, or none.
pub trait DefaultRolloutDigestRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRolloutDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutDigestTimezoneLengthRule {
    pub max_length: i64,
}

impl DefaultRolloutDigestRule for DefaultRolloutDigestTimezoneLengthRule {
    fn check(&self, item: &DefaultRolloutDigest) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutDigestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl DefaultRolloutDigestRule for DefaultRolloutDigestAllowedLabelRule {
    fn check(&self, item: &DefaultRolloutDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutDigestCodePrefixRule {
    pub prefix: String,
}

impl DefaultRolloutDigestRule for DefaultRolloutDigestCodePrefixRule {
    fn check(&self, item: &DefaultRolloutDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutDigestRequiredTimezoneRule {
    pub message: String,
}

impl DefaultRolloutDigestRule for DefaultRolloutDigestRequiredTimezoneRule {
    fn check(&self, item: &DefaultRolloutDigest) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutDigestLabelLengthRule {
    pub max_length: i64,
}

impl DefaultRolloutDigestRule for DefaultRolloutDigestLabelLengthRule {
    fn check(&self, item: &DefaultRolloutDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rollout digest.
#[derive(Debug, Clone)]
pub struct DefaultRolloutDigestValidator {
    pub rules: Vec<Rc<dyn DefaultRolloutDigestRule>>,
}

impl DefaultRolloutDigestValidator {
    pub fn validate(&self, item: &DefaultRolloutDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRolloutDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRolloutDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rollout_digest_validator() -> DefaultRolloutDigestValidator {
    DefaultRolloutDigestValidator {
        rules: vec![
            Rc::new(DefaultRolloutDigestTimezoneLengthRule { max_length: 16 }) as Rc<dyn DefaultRolloutDigestRule>,
            Rc::new(DefaultRolloutDigestAllowedLabelRule {
                allowed: vec!["fallback".to_string(), "rolling".to_string()],
            }),
            Rc::new(DefaultRolloutDigestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultRolloutDigestRequiredTimezoneRule {
                message: "default rollout digest needs an entry".to_string(),
            }),
            Rc::new(DefaultRolloutDigestLabelLengthRule { max_length: 86400 }),
        ],
    }
}
