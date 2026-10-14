use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default postmortem bundle must pass. Check returns its problems, or none.
pub trait DefaultPostmortemBundleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPostmortemBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPostmortemBundleFingerprintLengthRule {
    pub max_length: i64,
}

impl DefaultPostmortemBundleRule for DefaultPostmortemBundleFingerprintLengthRule {
    fn check(&self, item: &DefaultPostmortemBundle) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPostmortemBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultPostmortemBundleRule for DefaultPostmortemBundleAllowedCodeRule {
    fn check(&self, item: &DefaultPostmortemBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPostmortemBundleNameLengthRule {
    pub max_length: i64,
}

impl DefaultPostmortemBundleRule for DefaultPostmortemBundleNameLengthRule {
    fn check(&self, item: &DefaultPostmortemBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPostmortemBundleCodePrefixRule {
    pub prefix: String,
}

impl DefaultPostmortemBundleRule for DefaultPostmortemBundleCodePrefixRule {
    fn check(&self, item: &DefaultPostmortemBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPostmortemBundleCodeLengthRule {
    pub max_length: i64,
}

impl DefaultPostmortemBundleRule for DefaultPostmortemBundleCodeLengthRule {
    fn check(&self, item: &DefaultPostmortemBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default postmortem bundle.
#[derive(Debug, Clone)]
pub struct DefaultPostmortemBundleValidator {
    pub rules: Vec<Rc<dyn DefaultPostmortemBundleRule>>,
}

impl DefaultPostmortemBundleValidator {
    pub fn validate(&self, item: &DefaultPostmortemBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultPostmortemBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPostmortemBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_postmortem_bundle_validator() -> DefaultPostmortemBundleValidator {
    DefaultPostmortemBundleValidator {
        rules: vec![
            Rc::new(DefaultPostmortemBundleFingerprintLengthRule { max_length: 86400 })
                as Rc<dyn DefaultPostmortemBundleRule>,
            Rc::new(DefaultPostmortemBundleAllowedCodeRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(DefaultPostmortemBundleNameLengthRule { max_length: 32 }),
            Rc::new(DefaultPostmortemBundleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultPostmortemBundleCodeLengthRule { max_length: 120 }),
        ],
    }
}
