use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback egress threshold must pass. Check returns its problems, or none.
pub trait FallbackEgressThresholdRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEgressThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressThresholdRequiredNameRule {
    pub message: String,
}

impl FallbackEgressThresholdRule for FallbackEgressThresholdRequiredNameRule {
    fn check(&self, item: &FallbackEgressThreshold) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressThresholdLabelLengthRule {
    pub max_length: i64,
}

impl FallbackEgressThresholdRule for FallbackEgressThresholdLabelLengthRule {
    fn check(&self, item: &FallbackEgressThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressThresholdNameLengthRule {
    pub max_length: i64,
}

impl FallbackEgressThresholdRule for FallbackEgressThresholdNameLengthRule {
    fn check(&self, item: &FallbackEgressThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressThresholdCodeLengthRule {
    pub max_length: i64,
}

impl FallbackEgressThresholdRule for FallbackEgressThresholdCodeLengthRule {
    fn check(&self, item: &FallbackEgressThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackEgressThresholdRule for FallbackEgressThresholdAllowedCodeRule {
    fn check(&self, item: &FallbackEgressThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback egress threshold.
#[derive(Debug, Clone)]
pub struct FallbackEgressThresholdValidator {
    pub rules: Vec<Rc<dyn FallbackEgressThresholdRule>>,
}

impl FallbackEgressThresholdValidator {
    pub fn validate(&self, item: &FallbackEgressThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackEgressThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEgressThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_egress_threshold_validator() -> FallbackEgressThresholdValidator {
    FallbackEgressThresholdValidator {
        rules: vec![
            Rc::new(FallbackEgressThresholdRequiredNameRule {
                message: "fallback egress threshold needs this field".to_string(),
            }) as Rc<dyn FallbackEgressThresholdRule>,
            Rc::new(FallbackEgressThresholdLabelLengthRule { max_length: 120 }),
            Rc::new(FallbackEgressThresholdNameLengthRule { max_length: 3600 }),
            Rc::new(FallbackEgressThresholdCodeLengthRule { max_length: 255 }),
            Rc::new(FallbackEgressThresholdAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "regional".to_string()],
            }),
        ],
    }
}
