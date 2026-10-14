use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert bundle must pass. Check returns its problems, or none.
pub trait AlertBundleRule: std::fmt::Debug {
    fn check(&self, item: &AlertBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBundleRequiredCodeRule {
    pub message: String,
}

impl AlertBundleRule for AlertBundleRequiredCodeRule {
    fn check(&self, item: &AlertBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBundleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertBundleRule for AlertBundleThresholdRangeRule {
    fn check(&self, item: &AlertBundle) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBundleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertBundleRule for AlertBundleEndsAtRangeRule {
    fn check(&self, item: &AlertBundle) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBundleCodeLengthRule {
    pub max_length: i64,
}

impl AlertBundleRule for AlertBundleCodeLengthRule {
    fn check(&self, item: &AlertBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AlertBundleRule for AlertBundleAllowedCodeRule {
    fn check(&self, item: &AlertBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBundleCodePrefixRule {
    pub prefix: String,
}

impl AlertBundleRule for AlertBundleCodePrefixRule {
    fn check(&self, item: &AlertBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert bundle.
#[derive(Debug, Clone)]
pub struct AlertBundleValidator {
    pub rules: Vec<Rc<dyn AlertBundleRule>>,
}

impl AlertBundleValidator {
    pub fn validate(&self, item: &AlertBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_bundle_validator() -> AlertBundleValidator {
    AlertBundleValidator {
        rules: vec![
            Rc::new(AlertBundleRequiredCodeRule {
                message: "alert bundle needs this field".to_string(),
            }) as Rc<dyn AlertBundleRule>,
            Rc::new(AlertBundleThresholdRangeRule { min: 1.2, max: 4693.8 }),
            Rc::new(AlertBundleEndsAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(AlertBundleCodeLengthRule { max_length: 32 }),
            Rc::new(AlertBundleAllowedCodeRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(AlertBundleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
