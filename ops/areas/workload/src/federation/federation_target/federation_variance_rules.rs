use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation variance must pass. Check returns its problems, or none.
pub trait FederationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &FederationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationVarianceCodePrefixRule {
    pub prefix: String,
}

impl FederationVarianceRule for FederationVarianceCodePrefixRule {
    fn check(&self, item: &FederationVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationVarianceCodeLengthRule {
    pub max_length: i64,
}

impl FederationVarianceRule for FederationVarianceCodeLengthRule {
    fn check(&self, item: &FederationVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationVarianceNameLengthRule {
    pub max_length: i64,
}

impl FederationVarianceRule for FederationVarianceNameLengthRule {
    fn check(&self, item: &FederationVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationVarianceRequiredCodeRule {
    pub message: String,
}

impl FederationVarianceRule for FederationVarianceRequiredCodeRule {
    fn check(&self, item: &FederationVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation variance.
#[derive(Debug, Clone)]
pub struct FederationVarianceValidator {
    pub rules: Vec<Rc<dyn FederationVarianceRule>>,
}

impl FederationVarianceValidator {
    pub fn validate(&self, item: &FederationVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_variance_validator() -> FederationVarianceValidator {
    FederationVarianceValidator {
        rules: vec![
            Rc::new(FederationVarianceCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn FederationVarianceRule>,
            Rc::new(FederationVarianceCodeLengthRule { max_length: 120 }),
            Rc::new(FederationVarianceNameLengthRule { max_length: 16 }),
            Rc::new(FederationVarianceRequiredCodeRule {
                message: "federation variance needs a value".to_string(),
            }),
        ],
    }
}
