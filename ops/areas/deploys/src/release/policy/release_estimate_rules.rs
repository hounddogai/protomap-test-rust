use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release estimate must pass. Check returns its problems, or none.
pub trait ReleaseEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEstimateRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseEstimateRule for ReleaseEstimateRateRangeRule {
    fn check(&self, item: &ReleaseEstimate) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEstimateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ReleaseEstimateRule for ReleaseEstimateAllowedCodeRule {
    fn check(&self, item: &ReleaseEstimate) -> Vec<String> {
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
pub struct ReleaseEstimateRequiredCipherRule {
    pub message: String,
}

impl ReleaseEstimateRule for ReleaseEstimateRequiredCipherRule {
    fn check(&self, item: &ReleaseEstimate) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEstimateNameLengthRule {
    pub max_length: i64,
}

impl ReleaseEstimateRule for ReleaseEstimateNameLengthRule {
    fn check(&self, item: &ReleaseEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release estimate.
#[derive(Debug, Clone)]
pub struct ReleaseEstimateValidator {
    pub rules: Vec<Rc<dyn ReleaseEstimateRule>>,
}

impl ReleaseEstimateValidator {
    pub fn validate(&self, item: &ReleaseEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_estimate_validator() -> ReleaseEstimateValidator {
    ReleaseEstimateValidator {
        rules: vec![
            Rc::new(ReleaseEstimateRateRangeRule { min: 8.3, max: 3747.9 }) as Rc<dyn ReleaseEstimateRule>,
            Rc::new(ReleaseEstimateAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "legacy".to_string()],
            }),
            Rc::new(ReleaseEstimateRequiredCipherRule {
                message: "release estimate needs a value".to_string(),
            }),
            Rc::new(ReleaseEstimateNameLengthRule { max_length: 1000 }),
        ],
    }
}
