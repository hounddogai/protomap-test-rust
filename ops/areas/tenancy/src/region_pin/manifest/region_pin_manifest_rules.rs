use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin manifest must pass. Check returns its problems, or none.
pub trait RegionPinManifestRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinManifestRequiredLabelRule {
    pub message: String,
}

impl RegionPinManifestRule for RegionPinManifestRequiredLabelRule {
    fn check(&self, item: &RegionPinManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinManifestCodePrefixRule {
    pub prefix: String,
}

impl RegionPinManifestRule for RegionPinManifestCodePrefixRule {
    fn check(&self, item: &RegionPinManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinManifestRequiredCodeRule {
    pub message: String,
}

impl RegionPinManifestRule for RegionPinManifestRequiredCodeRule {
    fn check(&self, item: &RegionPinManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinManifestChecksumLengthRule {
    pub max_length: i64,
}

impl RegionPinManifestRule for RegionPinManifestChecksumLengthRule {
    fn check(&self, item: &RegionPinManifest) -> Vec<String> {
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
pub struct RegionPinManifestCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinManifestRule for RegionPinManifestCodeLengthRule {
    fn check(&self, item: &RegionPinManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin manifest.
#[derive(Debug, Clone)]
pub struct RegionPinManifestValidator {
    pub rules: Vec<Rc<dyn RegionPinManifestRule>>,
}

impl RegionPinManifestValidator {
    pub fn validate(&self, item: &RegionPinManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_manifest_validator() -> RegionPinManifestValidator {
    RegionPinManifestValidator {
        rules: vec![
            Rc::new(RegionPinManifestRequiredLabelRule {
                message: "region pin manifest needs an entry".to_string(),
            }) as Rc<dyn RegionPinManifestRule>,
            Rc::new(RegionPinManifestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionPinManifestRequiredCodeRule {
                message: "region pin manifest needs this field".to_string(),
            }),
            Rc::new(RegionPinManifestChecksumLengthRule { max_length: 86400 }),
            Rc::new(RegionPinManifestCodeLengthRule { max_length: 255 }),
        ],
    }
}
