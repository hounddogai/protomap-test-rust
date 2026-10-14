use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink bundle must pass. Check returns its problems, or none.
pub trait SinkBundleRule: std::fmt::Debug {
    fn check(&self, item: &SinkBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBundleRequiredCountryRule {
    pub message: String,
}

impl SinkBundleRule for SinkBundleRequiredCountryRule {
    fn check(&self, item: &SinkBundle) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBundleRequiredCodeRule {
    pub message: String,
}

impl SinkBundleRule for SinkBundleRequiredCodeRule {
    fn check(&self, item: &SinkBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBundleRequiredNameRule {
    pub message: String,
}

impl SinkBundleRule for SinkBundleRequiredNameRule {
    fn check(&self, item: &SinkBundle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBundleCountryLengthRule {
    pub max_length: i64,
}

impl SinkBundleRule for SinkBundleCountryLengthRule {
    fn check(&self, item: &SinkBundle) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink bundle.
#[derive(Debug, Clone)]
pub struct SinkBundleValidator {
    pub rules: Vec<Rc<dyn SinkBundleRule>>,
}

impl SinkBundleValidator {
    pub fn validate(&self, item: &SinkBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_bundle_validator() -> SinkBundleValidator {
    SinkBundleValidator {
        rules: vec![
            Rc::new(SinkBundleRequiredCountryRule {
                message: "sink bundle needs an entry".to_string(),
            }) as Rc<dyn SinkBundleRule>,
            Rc::new(SinkBundleRequiredCodeRule {
                message: "sink bundle needs a value".to_string(),
            }),
            Rc::new(SinkBundleRequiredNameRule {
                message: "sink bundle needs this field".to_string(),
            }),
            Rc::new(SinkBundleCountryLengthRule { max_length: 16 }),
        ],
    }
}
