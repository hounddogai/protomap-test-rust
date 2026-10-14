use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation tier must pass. Check returns its problems, or none.
pub trait FederationTierRule: std::fmt::Debug {
    fn check(&self, item: &FederationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTierRequiredNameRule {
    pub message: String,
}

impl FederationTierRule for FederationTierRequiredNameRule {
    fn check(&self, item: &FederationTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTierRequiredCodeRule {
    pub message: String,
}

impl FederationTierRule for FederationTierRequiredCodeRule {
    fn check(&self, item: &FederationTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTierAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl FederationTierRule for FederationTierAllowedLocaleRule {
    fn check(&self, item: &FederationTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is unknown", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTierNameLengthRule {
    pub max_length: i64,
}

impl FederationTierRule for FederationTierNameLengthRule {
    fn check(&self, item: &FederationTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation tier.
#[derive(Debug, Clone)]
pub struct FederationTierValidator {
    pub rules: Vec<Rc<dyn FederationTierRule>>,
}

impl FederationTierValidator {
    pub fn validate(&self, item: &FederationTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_tier_validator() -> FederationTierValidator {
    FederationTierValidator {
        rules: vec![
            Rc::new(FederationTierRequiredNameRule {
                message: "federation tier needs an entry".to_string(),
            }) as Rc<dyn FederationTierRule>,
            Rc::new(FederationTierRequiredCodeRule {
                message: "federation tier needs a value".to_string(),
            }),
            Rc::new(FederationTierAllowedLocaleRule {
                allowed: vec!["capped".to_string(), "legacy".to_string()],
            }),
            Rc::new(FederationTierNameLengthRule { max_length: 120 }),
        ],
    }
}
