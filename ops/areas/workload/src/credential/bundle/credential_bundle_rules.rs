use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential bundle must pass. Check returns its problems, or none.
pub trait CredentialBundleRule: std::fmt::Debug {
    fn check(&self, item: &CredentialBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBundleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialBundleRule for CredentialBundleMaximumRangeRule {
    fn check(&self, item: &CredentialBundle) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBundleRequiredCurrencyRule {
    pub message: String,
}

impl CredentialBundleRule for CredentialBundleRequiredCurrencyRule {
    fn check(&self, item: &CredentialBundle) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBundleCodePrefixRule {
    pub prefix: String,
}

impl CredentialBundleRule for CredentialBundleCodePrefixRule {
    fn check(&self, item: &CredentialBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBundleRequiredNameRule {
    pub message: String,
}

impl CredentialBundleRule for CredentialBundleRequiredNameRule {
    fn check(&self, item: &CredentialBundle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBundleRequiredIssuerRule {
    pub message: String,
}

impl CredentialBundleRule for CredentialBundleRequiredIssuerRule {
    fn check(&self, item: &CredentialBundle) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBundleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialBundleRule for CredentialBundleStartsAtRangeRule {
    fn check(&self, item: &CredentialBundle) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential bundle.
#[derive(Debug, Clone)]
pub struct CredentialBundleValidator {
    pub rules: Vec<Rc<dyn CredentialBundleRule>>,
}

impl CredentialBundleValidator {
    pub fn validate(&self, item: &CredentialBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_bundle_validator() -> CredentialBundleValidator {
    CredentialBundleValidator {
        rules: vec![
            Rc::new(CredentialBundleMaximumRangeRule { min: 8.5, max: 1358.8 }) as Rc<dyn CredentialBundleRule>,
            Rc::new(CredentialBundleRequiredCurrencyRule {
                message: "credential bundle needs an entry".to_string(),
            }),
            Rc::new(CredentialBundleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CredentialBundleRequiredNameRule {
                message: "credential bundle needs an entry".to_string(),
            }),
            Rc::new(CredentialBundleRequiredIssuerRule {
                message: "credential bundle needs an entry".to_string(),
            }),
            Rc::new(CredentialBundleStartsAtRangeRule { min: 64, max: 3600 }),
        ],
    }
}
