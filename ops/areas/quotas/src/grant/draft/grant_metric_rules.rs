use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant metric must pass. Check returns its problems, or none.
pub trait GrantMetricRule: std::fmt::Debug {
    fn check(&self, item: &GrantMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMetricCodePrefixRule {
    pub prefix: String,
}

impl GrantMetricRule for GrantMetricCodePrefixRule {
    fn check(&self, item: &GrantMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMetricRequiredCodeRule {
    pub message: String,
}

impl GrantMetricRule for GrantMetricRequiredCodeRule {
    fn check(&self, item: &GrantMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMetricCountryLengthRule {
    pub max_length: i64,
}

impl GrantMetricRule for GrantMetricCountryLengthRule {
    fn check(&self, item: &GrantMetric) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMetricRequiredChecksumRule {
    pub message: String,
}

impl GrantMetricRule for GrantMetricRequiredChecksumRule {
    fn check(&self, item: &GrantMetric) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantMetricChecksumLengthRule {
    pub max_length: i64,
}

impl GrantMetricRule for GrantMetricChecksumLengthRule {
    fn check(&self, item: &GrantMetric) -> Vec<String> {
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
pub struct GrantMetricCodeLengthRule {
    pub max_length: i64,
}

impl GrantMetricRule for GrantMetricCodeLengthRule {
    fn check(&self, item: &GrantMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant metric.
#[derive(Debug, Clone)]
pub struct GrantMetricValidator {
    pub rules: Vec<Rc<dyn GrantMetricRule>>,
}

impl GrantMetricValidator {
    pub fn validate(&self, item: &GrantMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_metric_validator() -> GrantMetricValidator {
    GrantMetricValidator {
        rules: vec![
            Rc::new(GrantMetricCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn GrantMetricRule>,
            Rc::new(GrantMetricRequiredCodeRule {
                message: "grant metric needs a value".to_string(),
            }),
            Rc::new(GrantMetricCountryLengthRule { max_length: 120 }),
            Rc::new(GrantMetricRequiredChecksumRule {
                message: "grant metric needs a value".to_string(),
            }),
            Rc::new(GrantMetricChecksumLengthRule { max_length: 80 }),
            Rc::new(GrantMetricCodeLengthRule { max_length: 120 }),
        ],
    }
}
