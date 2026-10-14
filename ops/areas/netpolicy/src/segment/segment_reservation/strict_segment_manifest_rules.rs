use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict segment manifest must pass. Check returns its problems, or none.
pub trait StrictSegmentManifestRule: std::fmt::Debug {
    fn check(&self, item: &StrictSegmentManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentManifestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl StrictSegmentManifestRule for StrictSegmentManifestAllowedLabelRule {
    fn check(&self, item: &StrictSegmentManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentManifestCodePrefixRule {
    pub prefix: String,
}

impl StrictSegmentManifestRule for StrictSegmentManifestCodePrefixRule {
    fn check(&self, item: &StrictSegmentManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentManifestLabelLengthRule {
    pub max_length: i64,
}

impl StrictSegmentManifestRule for StrictSegmentManifestLabelLengthRule {
    fn check(&self, item: &StrictSegmentManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict segment manifest.
#[derive(Debug, Clone)]
pub struct StrictSegmentManifestValidator {
    pub rules: Vec<Rc<dyn StrictSegmentManifestRule>>,
}

impl StrictSegmentManifestValidator {
    pub fn validate(&self, item: &StrictSegmentManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictSegmentManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSegmentManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_segment_manifest_validator() -> StrictSegmentManifestValidator {
    StrictSegmentManifestValidator {
        rules: vec![
            Rc::new(StrictSegmentManifestAllowedLabelRule {
                allowed: vec!["strict".to_string()],
            }) as Rc<dyn StrictSegmentManifestRule>,
            Rc::new(StrictSegmentManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictSegmentManifestLabelLengthRule { max_length: 86400 }),
        ],
    }
}
