use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem bundle must pass. Check returns its problems, or none.
pub trait PostmortemBundleRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBundleCodePrefixRule {
    pub prefix: String,
}

impl PostmortemBundleRule for PostmortemBundleCodePrefixRule {
    fn check(&self, item: &PostmortemBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBundleCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemBundleRule for PostmortemBundleCodeLengthRule {
    fn check(&self, item: &PostmortemBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBundleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemBundleRule for PostmortemBundleWindowDaysRangeRule {
    fn check(&self, item: &PostmortemBundle) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBundleRequiredCodeRule {
    pub message: String,
}

impl PostmortemBundleRule for PostmortemBundleRequiredCodeRule {
    fn check(&self, item: &PostmortemBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBundleRequiredChannelRule {
    pub message: String,
}

impl PostmortemBundleRule for PostmortemBundleRequiredChannelRule {
    fn check(&self, item: &PostmortemBundle) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem bundle.
#[derive(Debug, Clone)]
pub struct PostmortemBundleValidator {
    pub rules: Vec<Rc<dyn PostmortemBundleRule>>,
}

impl PostmortemBundleValidator {
    pub fn validate(&self, item: &PostmortemBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_bundle_validator() -> PostmortemBundleValidator {
    PostmortemBundleValidator {
        rules: vec![
            Rc::new(PostmortemBundleCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn PostmortemBundleRule>,
            Rc::new(PostmortemBundleCodeLengthRule { max_length: 16 }),
            Rc::new(PostmortemBundleWindowDaysRangeRule { min: 255, max: 255 }),
            Rc::new(PostmortemBundleRequiredCodeRule {
                message: "postmortem bundle needs this field".to_string(),
            }),
            Rc::new(PostmortemBundleRequiredChannelRule {
                message: "postmortem bundle needs a value".to_string(),
            }),
        ],
    }
}
