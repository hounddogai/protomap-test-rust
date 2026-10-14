use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation bundle must pass. Check returns its problems, or none.
pub trait DelegationBundleRule: std::fmt::Debug {
    fn check(&self, item: &DelegationBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBundleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationBundleRule for DelegationBundleWindowDaysRangeRule {
    fn check(&self, item: &DelegationBundle) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBundleCodeLengthRule {
    pub max_length: i64,
}

impl DelegationBundleRule for DelegationBundleCodeLengthRule {
    fn check(&self, item: &DelegationBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBundleRequiredDescriptionRule {
    pub message: String,
}

impl DelegationBundleRule for DelegationBundleRequiredDescriptionRule {
    fn check(&self, item: &DelegationBundle) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBundleRequiredCodeRule {
    pub message: String,
}

impl DelegationBundleRule for DelegationBundleRequiredCodeRule {
    fn check(&self, item: &DelegationBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBundleLabelLengthRule {
    pub max_length: i64,
}

impl DelegationBundleRule for DelegationBundleLabelLengthRule {
    fn check(&self, item: &DelegationBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation bundle.
#[derive(Debug, Clone)]
pub struct DelegationBundleValidator {
    pub rules: Vec<Rc<dyn DelegationBundleRule>>,
}

impl DelegationBundleValidator {
    pub fn validate(&self, item: &DelegationBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_bundle_validator() -> DelegationBundleValidator {
    DelegationBundleValidator {
        rules: vec![
            Rc::new(DelegationBundleWindowDaysRangeRule { min: 120, max: 86400 }) as Rc<dyn DelegationBundleRule>,
            Rc::new(DelegationBundleCodeLengthRule { max_length: 16 }),
            Rc::new(DelegationBundleRequiredDescriptionRule {
                message: "delegation bundle needs this field".to_string(),
            }),
            Rc::new(DelegationBundleRequiredCodeRule {
                message: "delegation bundle needs an entry".to_string(),
            }),
            Rc::new(DelegationBundleLabelLengthRule { max_length: 16 }),
        ],
    }
}
