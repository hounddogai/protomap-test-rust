use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual region pin bundle must pass. Check returns its problems, or none.
pub trait ManualRegionPinBundleRule: std::fmt::Debug {
    fn check(&self, item: &ManualRegionPinBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinBundleRequiredExternalRefRule {
    pub message: String,
}

impl ManualRegionPinBundleRule for ManualRegionPinBundleRequiredExternalRefRule {
    fn check(&self, item: &ManualRegionPinBundle) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinBundleNameLengthRule {
    pub max_length: i64,
}

impl ManualRegionPinBundleRule for ManualRegionPinBundleNameLengthRule {
    fn check(&self, item: &ManualRegionPinBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinBundleRequiredCodeRule {
    pub message: String,
}

impl ManualRegionPinBundleRule for ManualRegionPinBundleRequiredCodeRule {
    fn check(&self, item: &ManualRegionPinBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinBundleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ManualRegionPinBundleRule for ManualRegionPinBundleAllowedLabelRule {
    fn check(&self, item: &ManualRegionPinBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinBundleExternalRefLengthRule {
    pub max_length: i64,
}

impl ManualRegionPinBundleRule for ManualRegionPinBundleExternalRefLengthRule {
    fn check(&self, item: &ManualRegionPinBundle) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual region pin bundle.
#[derive(Debug, Clone)]
pub struct ManualRegionPinBundleValidator {
    pub rules: Vec<Rc<dyn ManualRegionPinBundleRule>>,
}

impl ManualRegionPinBundleValidator {
    pub fn validate(&self, item: &ManualRegionPinBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualRegionPinBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRegionPinBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_region_pin_bundle_validator() -> ManualRegionPinBundleValidator {
    ManualRegionPinBundleValidator {
        rules: vec![
            Rc::new(ManualRegionPinBundleRequiredExternalRefRule {
                message: "manual region pin bundle needs an entry".to_string(),
            }) as Rc<dyn ManualRegionPinBundleRule>,
            Rc::new(ManualRegionPinBundleNameLengthRule { max_length: 16 }),
            Rc::new(ManualRegionPinBundleRequiredCodeRule {
                message: "manual region pin bundle needs this field".to_string(),
            }),
            Rc::new(ManualRegionPinBundleAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(ManualRegionPinBundleExternalRefLengthRule { max_length: 64 }),
        ],
    }
}
