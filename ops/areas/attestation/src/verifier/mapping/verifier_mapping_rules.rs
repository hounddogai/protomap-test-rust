use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier mapping must pass. Check returns its problems, or none.
pub trait VerifierMappingRule: std::fmt::Debug {
    fn check(&self, item: &VerifierMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMappingRequiredCodeRule {
    pub message: String,
}

impl VerifierMappingRule for VerifierMappingRequiredCodeRule {
    fn check(&self, item: &VerifierMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMappingRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierMappingRule for VerifierMappingRetriesRangeRule {
    fn check(&self, item: &VerifierMapping) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMappingCodeLengthRule {
    pub max_length: i64,
}

impl VerifierMappingRule for VerifierMappingCodeLengthRule {
    fn check(&self, item: &VerifierMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMappingCodePrefixRule {
    pub prefix: String,
}

impl VerifierMappingRule for VerifierMappingCodePrefixRule {
    fn check(&self, item: &VerifierMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMappingRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierMappingRule for VerifierMappingRateRangeRule {
    fn check(&self, item: &VerifierMapping) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMappingLocaleLengthRule {
    pub max_length: i64,
}

impl VerifierMappingRule for VerifierMappingLocaleLengthRule {
    fn check(&self, item: &VerifierMapping) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier mapping.
#[derive(Debug, Clone)]
pub struct VerifierMappingValidator {
    pub rules: Vec<Rc<dyn VerifierMappingRule>>,
}

impl VerifierMappingValidator {
    pub fn validate(&self, item: &VerifierMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_mapping_validator() -> VerifierMappingValidator {
    VerifierMappingValidator {
        rules: vec![
            Rc::new(VerifierMappingRequiredCodeRule {
                message: "verifier mapping needs this field".to_string(),
            }) as Rc<dyn VerifierMappingRule>,
            Rc::new(VerifierMappingRetriesRangeRule { min: 86400, max: 86400 }),
            Rc::new(VerifierMappingCodeLengthRule { max_length: 16 }),
            Rc::new(VerifierMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(VerifierMappingRateRangeRule { min: 9.3, max: 4681.2 }),
            Rc::new(VerifierMappingLocaleLengthRule { max_length: 3600 }),
        ],
    }
}
