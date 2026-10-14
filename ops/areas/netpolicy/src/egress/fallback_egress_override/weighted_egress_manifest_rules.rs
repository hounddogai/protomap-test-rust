use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted egress manifest must pass. Check returns its problems, or none.
pub trait WeightedEgressManifestRule: std::fmt::Debug {
    fn check(&self, item: &WeightedEgressManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifestPrincipalLengthRule {
    pub max_length: i64,
}

impl WeightedEgressManifestRule for WeightedEgressManifestPrincipalLengthRule {
    fn check(&self, item: &WeightedEgressManifest) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl WeightedEgressManifestRule for WeightedEgressManifestAllowedTitleRule {
    fn check(&self, item: &WeightedEgressManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifestRequiredPrincipalRule {
    pub message: String,
}

impl WeightedEgressManifestRule for WeightedEgressManifestRequiredPrincipalRule {
    fn check(&self, item: &WeightedEgressManifest) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifestRequiredChannelRule {
    pub message: String,
}

impl WeightedEgressManifestRule for WeightedEgressManifestRequiredChannelRule {
    fn check(&self, item: &WeightedEgressManifest) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifestChannelLengthRule {
    pub max_length: i64,
}

impl WeightedEgressManifestRule for WeightedEgressManifestChannelLengthRule {
    fn check(&self, item: &WeightedEgressManifest) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifestCodePrefixRule {
    pub prefix: String,
}

impl WeightedEgressManifestRule for WeightedEgressManifestCodePrefixRule {
    fn check(&self, item: &WeightedEgressManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted egress manifest.
#[derive(Debug, Clone)]
pub struct WeightedEgressManifestValidator {
    pub rules: Vec<Rc<dyn WeightedEgressManifestRule>>,
}

impl WeightedEgressManifestValidator {
    pub fn validate(&self, item: &WeightedEgressManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedEgressManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedEgressManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_egress_manifest_validator() -> WeightedEgressManifestValidator {
    WeightedEgressManifestValidator {
        rules: vec![
            Rc::new(WeightedEgressManifestPrincipalLengthRule { max_length: 32 }) as Rc<dyn WeightedEgressManifestRule>,
            Rc::new(WeightedEgressManifestAllowedTitleRule {
                allowed: vec!["default".to_string(), "tiered".to_string(), "tiered".to_string()],
            }),
            Rc::new(WeightedEgressManifestRequiredPrincipalRule {
                message: "weighted egress manifest needs a value".to_string(),
            }),
            Rc::new(WeightedEgressManifestRequiredChannelRule {
                message: "weighted egress manifest needs an entry".to_string(),
            }),
            Rc::new(WeightedEgressManifestChannelLengthRule { max_length: 3600 }),
            Rc::new(WeightedEgressManifestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
