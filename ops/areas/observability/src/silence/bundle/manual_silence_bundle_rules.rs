use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual silence bundle must pass. Check returns its problems, or none.
pub trait ManualSilenceBundleRule: std::fmt::Debug {
    fn check(&self, item: &ManualSilenceBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundleCodeLengthRule {
    pub max_length: i64,
}

impl ManualSilenceBundleRule for ManualSilenceBundleCodeLengthRule {
    fn check(&self, item: &ManualSilenceBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundleAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl ManualSilenceBundleRule for ManualSilenceBundleAllowedTimezoneRule {
    fn check(&self, item: &ManualSilenceBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is not allowed", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundleLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualSilenceBundleRule for ManualSilenceBundleLimitRangeRule {
    fn check(&self, item: &ManualSilenceBundle) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundleRequiredCodeRule {
    pub message: String,
}

impl ManualSilenceBundleRule for ManualSilenceBundleRequiredCodeRule {
    fn check(&self, item: &ManualSilenceBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundleRequiredTimezoneRule {
    pub message: String,
}

impl ManualSilenceBundleRule for ManualSilenceBundleRequiredTimezoneRule {
    fn check(&self, item: &ManualSilenceBundle) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundleRequiredIssuerRule {
    pub message: String,
}

impl ManualSilenceBundleRule for ManualSilenceBundleRequiredIssuerRule {
    fn check(&self, item: &ManualSilenceBundle) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual silence bundle.
#[derive(Debug, Clone)]
pub struct ManualSilenceBundleValidator {
    pub rules: Vec<Rc<dyn ManualSilenceBundleRule>>,
}

impl ManualSilenceBundleValidator {
    pub fn validate(&self, item: &ManualSilenceBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualSilenceBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSilenceBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_silence_bundle_validator() -> ManualSilenceBundleValidator {
    ManualSilenceBundleValidator {
        rules: vec![
            Rc::new(ManualSilenceBundleCodeLengthRule { max_length: 3600 }) as Rc<dyn ManualSilenceBundleRule>,
            Rc::new(ManualSilenceBundleAllowedTimezoneRule {
                allowed: vec!["seasonal".to_string(), "scheduled".to_string()],
            }),
            Rc::new(ManualSilenceBundleLimitRangeRule { min: 32, max: 64 }),
            Rc::new(ManualSilenceBundleRequiredCodeRule {
                message: "manual silence bundle needs this field".to_string(),
            }),
            Rc::new(ManualSilenceBundleRequiredTimezoneRule {
                message: "manual silence bundle needs this field".to_string(),
            }),
            Rc::new(ManualSilenceBundleRequiredIssuerRule {
                message: "manual silence bundle needs this field".to_string(),
            }),
        ],
    }
}
