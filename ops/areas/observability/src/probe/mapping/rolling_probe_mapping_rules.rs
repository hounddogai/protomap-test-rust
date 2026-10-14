use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling probe mapping must pass. Check returns its problems, or none.
pub trait RollingProbeMappingRule: std::fmt::Debug {
    fn check(&self, item: &RollingProbeMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeMappingCodePrefixRule {
    pub prefix: String,
}

impl RollingProbeMappingRule for RollingProbeMappingCodePrefixRule {
    fn check(&self, item: &RollingProbeMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeMappingRequiredTitleRule {
    pub message: String,
}

impl RollingProbeMappingRule for RollingProbeMappingRequiredTitleRule {
    fn check(&self, item: &RollingProbeMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeMappingRequiredSegmentRule {
    pub message: String,
}

impl RollingProbeMappingRule for RollingProbeMappingRequiredSegmentRule {
    fn check(&self, item: &RollingProbeMapping) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeMappingFingerprintLengthRule {
    pub max_length: i64,
}

impl RollingProbeMappingRule for RollingProbeMappingFingerprintLengthRule {
    fn check(&self, item: &RollingProbeMapping) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling probe mapping.
#[derive(Debug, Clone)]
pub struct RollingProbeMappingValidator {
    pub rules: Vec<Rc<dyn RollingProbeMappingRule>>,
}

impl RollingProbeMappingValidator {
    pub fn validate(&self, item: &RollingProbeMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingProbeMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingProbeMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_probe_mapping_validator() -> RollingProbeMappingValidator {
    RollingProbeMappingValidator {
        rules: vec![
            Rc::new(RollingProbeMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RollingProbeMappingRule>,
            Rc::new(RollingProbeMappingRequiredTitleRule {
                message: "rolling probe mapping needs this field".to_string(),
            }),
            Rc::new(RollingProbeMappingRequiredSegmentRule {
                message: "rolling probe mapping needs this field".to_string(),
            }),
            Rc::new(RollingProbeMappingFingerprintLengthRule { max_length: 86400 }),
        ],
    }
}
