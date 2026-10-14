use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default usage manifest must pass. Check returns its problems, or none.
pub trait DefaultUsageManifestRule: std::fmt::Debug {
    fn check(&self, item: &DefaultUsageManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageManifestAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl DefaultUsageManifestRule for DefaultUsageManifestAllowedSourceRule {
    fn check(&self, item: &DefaultUsageManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is unknown", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageManifestNameLengthRule {
    pub max_length: i64,
}

impl DefaultUsageManifestRule for DefaultUsageManifestNameLengthRule {
    fn check(&self, item: &DefaultUsageManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageManifestRequiredFingerprintRule {
    pub message: String,
}

impl DefaultUsageManifestRule for DefaultUsageManifestRequiredFingerprintRule {
    fn check(&self, item: &DefaultUsageManifest) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageManifestRequiredNameRule {
    pub message: String,
}

impl DefaultUsageManifestRule for DefaultUsageManifestRequiredNameRule {
    fn check(&self, item: &DefaultUsageManifest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageManifestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultUsageManifestRule for DefaultUsageManifestMaximumRangeRule {
    fn check(&self, item: &DefaultUsageManifest) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default usage manifest.
#[derive(Debug, Clone)]
pub struct DefaultUsageManifestValidator {
    pub rules: Vec<Rc<dyn DefaultUsageManifestRule>>,
}

impl DefaultUsageManifestValidator {
    pub fn validate(&self, item: &DefaultUsageManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultUsageManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultUsageManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_usage_manifest_validator() -> DefaultUsageManifestValidator {
    DefaultUsageManifestValidator {
        rules: vec![
            Rc::new(DefaultUsageManifestAllowedSourceRule { allowed: Vec::new() }) as Rc<dyn DefaultUsageManifestRule>,
            Rc::new(DefaultUsageManifestNameLengthRule { max_length: 80 }),
            Rc::new(DefaultUsageManifestRequiredFingerprintRule {
                message: "default usage manifest needs this field".to_string(),
            }),
            Rc::new(DefaultUsageManifestRequiredNameRule {
                message: "default usage manifest needs an entry".to_string(),
            }),
            Rc::new(DefaultUsageManifestMaximumRangeRule { min: 6.4, max: 818.6 }),
        ],
    }
}
