use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional suspension bundle must pass. Check returns its problems, or none.
pub trait RegionalSuspensionBundleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSuspensionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBundleChecksumLengthRule {
    pub max_length: i64,
}

impl RegionalSuspensionBundleRule for RegionalSuspensionBundleChecksumLengthRule {
    fn check(&self, item: &RegionalSuspensionBundle) -> Vec<String> {
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
pub struct RegionalSuspensionBundleLabelLengthRule {
    pub max_length: i64,
}

impl RegionalSuspensionBundleRule for RegionalSuspensionBundleLabelLengthRule {
    fn check(&self, item: &RegionalSuspensionBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBundleRequiredLabelRule {
    pub message: String,
}

impl RegionalSuspensionBundleRule for RegionalSuspensionBundleRequiredLabelRule {
    fn check(&self, item: &RegionalSuspensionBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBundleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalSuspensionBundleRule for RegionalSuspensionBundleEndsAtRangeRule {
    fn check(&self, item: &RegionalSuspensionBundle) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBundleCodePrefixRule {
    pub prefix: String,
}

impl RegionalSuspensionBundleRule for RegionalSuspensionBundleCodePrefixRule {
    fn check(&self, item: &RegionalSuspensionBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional suspension bundle.
#[derive(Debug, Clone)]
pub struct RegionalSuspensionBundleValidator {
    pub rules: Vec<Rc<dyn RegionalSuspensionBundleRule>>,
}

impl RegionalSuspensionBundleValidator {
    pub fn validate(&self, item: &RegionalSuspensionBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalSuspensionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSuspensionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_suspension_bundle_validator() -> RegionalSuspensionBundleValidator {
    RegionalSuspensionBundleValidator {
        rules: vec![
            Rc::new(RegionalSuspensionBundleChecksumLengthRule { max_length: 86400 })
                as Rc<dyn RegionalSuspensionBundleRule>,
            Rc::new(RegionalSuspensionBundleLabelLengthRule { max_length: 32 }),
            Rc::new(RegionalSuspensionBundleRequiredLabelRule {
                message: "regional suspension bundle needs an entry".to_string(),
            }),
            Rc::new(RegionalSuspensionBundleEndsAtRangeRule { min: 16, max: 64 }),
            Rc::new(RegionalSuspensionBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
