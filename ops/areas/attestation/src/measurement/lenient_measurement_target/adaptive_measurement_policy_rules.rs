use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive measurement policy must pass. Check returns its problems, or none.
pub trait AdaptiveMeasurementPolicyRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveMeasurementPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementPolicyRequiredCodeRule {
    pub message: String,
}

impl AdaptiveMeasurementPolicyRule for AdaptiveMeasurementPolicyRequiredCodeRule {
    fn check(&self, item: &AdaptiveMeasurementPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementPolicyCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveMeasurementPolicyRule for AdaptiveMeasurementPolicyCodePrefixRule {
    fn check(&self, item: &AdaptiveMeasurementPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementPolicyRequiredExternalRefRule {
    pub message: String,
}

impl AdaptiveMeasurementPolicyRule for AdaptiveMeasurementPolicyRequiredExternalRefRule {
    fn check(&self, item: &AdaptiveMeasurementPolicy) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementPolicyRequiredIssuerRule {
    pub message: String,
}

impl AdaptiveMeasurementPolicyRule for AdaptiveMeasurementPolicyRequiredIssuerRule {
    fn check(&self, item: &AdaptiveMeasurementPolicy) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementPolicyCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementPolicyRule for AdaptiveMeasurementPolicyCodeLengthRule {
    fn check(&self, item: &AdaptiveMeasurementPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive measurement policy.
#[derive(Debug, Clone)]
pub struct AdaptiveMeasurementPolicyValidator {
    pub rules: Vec<Rc<dyn AdaptiveMeasurementPolicyRule>>,
}

impl AdaptiveMeasurementPolicyValidator {
    pub fn validate(&self, item: &AdaptiveMeasurementPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveMeasurementPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveMeasurementPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_measurement_policy_validator() -> AdaptiveMeasurementPolicyValidator {
    AdaptiveMeasurementPolicyValidator {
        rules: vec![
            Rc::new(AdaptiveMeasurementPolicyRequiredCodeRule {
                message: "adaptive measurement policy needs this field".to_string(),
            }) as Rc<dyn AdaptiveMeasurementPolicyRule>,
            Rc::new(AdaptiveMeasurementPolicyCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(AdaptiveMeasurementPolicyRequiredExternalRefRule {
                message: "adaptive measurement policy needs this field".to_string(),
            }),
            Rc::new(AdaptiveMeasurementPolicyRequiredIssuerRule {
                message: "adaptive measurement policy needs a value".to_string(),
            }),
            Rc::new(AdaptiveMeasurementPolicyCodeLengthRule { max_length: 80 }),
        ],
    }
}
