use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global verifier variance must pass. Check returns its problems, or none.
pub trait GlobalVerifierVarianceRule: std::fmt::Debug {
    fn check(&self, item: &GlobalVerifierVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVerifierVarianceFingerprintLengthRule {
    pub max_length: i64,
}

impl GlobalVerifierVarianceRule for GlobalVerifierVarianceFingerprintLengthRule {
    fn check(&self, item: &GlobalVerifierVariance) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVerifierVarianceCodePrefixRule {
    pub prefix: String,
}

impl GlobalVerifierVarianceRule for GlobalVerifierVarianceCodePrefixRule {
    fn check(&self, item: &GlobalVerifierVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVerifierVarianceRequiredFingerprintRule {
    pub message: String,
}

impl GlobalVerifierVarianceRule for GlobalVerifierVarianceRequiredFingerprintRule {
    fn check(&self, item: &GlobalVerifierVariance) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVerifierVarianceWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalVerifierVarianceRule for GlobalVerifierVarianceWeightRangeRule {
    fn check(&self, item: &GlobalVerifierVariance) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global verifier variance.
#[derive(Debug, Clone)]
pub struct GlobalVerifierVarianceValidator {
    pub rules: Vec<Rc<dyn GlobalVerifierVarianceRule>>,
}

impl GlobalVerifierVarianceValidator {
    pub fn validate(&self, item: &GlobalVerifierVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalVerifierVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalVerifierVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_verifier_variance_validator() -> GlobalVerifierVarianceValidator {
    GlobalVerifierVarianceValidator {
        rules: vec![
            Rc::new(GlobalVerifierVarianceFingerprintLengthRule { max_length: 64 })
                as Rc<dyn GlobalVerifierVarianceRule>,
            Rc::new(GlobalVerifierVarianceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GlobalVerifierVarianceRequiredFingerprintRule {
                message: "global verifier variance needs this field".to_string(),
            }),
            Rc::new(GlobalVerifierVarianceWeightRangeRule { min: 5.6, max: 4700.0 }),
        ],
    }
}
