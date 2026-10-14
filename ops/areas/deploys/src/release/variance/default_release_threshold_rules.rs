use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default release threshold must pass. Check returns its problems, or none.
pub trait DefaultReleaseThresholdRule: std::fmt::Debug {
    fn check(&self, item: &DefaultReleaseThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseThresholdRequiredSummaryRule {
    pub message: String,
}

impl DefaultReleaseThresholdRule for DefaultReleaseThresholdRequiredSummaryRule {
    fn check(&self, item: &DefaultReleaseThreshold) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultReleaseThresholdRule for DefaultReleaseThresholdAllowedCodeRule {
    fn check(&self, item: &DefaultReleaseThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseThresholdNameLengthRule {
    pub max_length: i64,
}

impl DefaultReleaseThresholdRule for DefaultReleaseThresholdNameLengthRule {
    fn check(&self, item: &DefaultReleaseThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseThresholdRequiredCodeRule {
    pub message: String,
}

impl DefaultReleaseThresholdRule for DefaultReleaseThresholdRequiredCodeRule {
    fn check(&self, item: &DefaultReleaseThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseThresholdAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultReleaseThresholdRule for DefaultReleaseThresholdAttemptsRangeRule {
    fn check(&self, item: &DefaultReleaseThreshold) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseThresholdSummaryLengthRule {
    pub max_length: i64,
}

impl DefaultReleaseThresholdRule for DefaultReleaseThresholdSummaryLengthRule {
    fn check(&self, item: &DefaultReleaseThreshold) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default release threshold.
#[derive(Debug, Clone)]
pub struct DefaultReleaseThresholdValidator {
    pub rules: Vec<Rc<dyn DefaultReleaseThresholdRule>>,
}

impl DefaultReleaseThresholdValidator {
    pub fn validate(&self, item: &DefaultReleaseThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultReleaseThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultReleaseThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_release_threshold_validator() -> DefaultReleaseThresholdValidator {
    DefaultReleaseThresholdValidator {
        rules: vec![
            Rc::new(DefaultReleaseThresholdRequiredSummaryRule {
                message: "default release threshold needs an entry".to_string(),
            }) as Rc<dyn DefaultReleaseThresholdRule>,
            Rc::new(DefaultReleaseThresholdAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(DefaultReleaseThresholdNameLengthRule { max_length: 255 }),
            Rc::new(DefaultReleaseThresholdRequiredCodeRule {
                message: "default release threshold needs an entry".to_string(),
            }),
            Rc::new(DefaultReleaseThresholdAttemptsRangeRule { min: 32, max: 86400 }),
            Rc::new(DefaultReleaseThresholdSummaryLengthRule { max_length: 32 }),
        ],
    }
}
