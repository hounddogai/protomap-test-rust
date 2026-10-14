use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding bundle must pass. Check returns its problems, or none.
pub trait BindingBundleRule: std::fmt::Debug {
    fn check(&self, item: &BindingBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBundleRequiredTitleRule {
    pub message: String,
}

impl BindingBundleRule for BindingBundleRequiredTitleRule {
    fn check(&self, item: &BindingBundle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBundleTitleLengthRule {
    pub max_length: i64,
}

impl BindingBundleRule for BindingBundleTitleLengthRule {
    fn check(&self, item: &BindingBundle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBundleCodePrefixRule {
    pub prefix: String,
}

impl BindingBundleRule for BindingBundleCodePrefixRule {
    fn check(&self, item: &BindingBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBundleVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingBundleRule for BindingBundleVersionRangeRule {
    fn check(&self, item: &BindingBundle) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding bundle.
#[derive(Debug, Clone)]
pub struct BindingBundleValidator {
    pub rules: Vec<Rc<dyn BindingBundleRule>>,
}

impl BindingBundleValidator {
    pub fn validate(&self, item: &BindingBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_bundle_validator() -> BindingBundleValidator {
    BindingBundleValidator {
        rules: vec![
            Rc::new(BindingBundleRequiredTitleRule {
                message: "binding bundle needs this field".to_string(),
            }) as Rc<dyn BindingBundleRule>,
            Rc::new(BindingBundleTitleLengthRule { max_length: 1000 }),
            Rc::new(BindingBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BindingBundleVersionRangeRule { min: 255, max: 255 }),
        ],
    }
}
