use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback delegation window must pass. Check returns its problems, or none.
pub trait FallbackDelegationWindowRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDelegationWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationWindowRequiredCodeRule {
    pub message: String,
}

impl FallbackDelegationWindowRule for FallbackDelegationWindowRequiredCodeRule {
    fn check(&self, item: &FallbackDelegationWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationWindowAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FallbackDelegationWindowRule for FallbackDelegationWindowAllowedLabelRule {
    fn check(&self, item: &FallbackDelegationWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationWindowRequiredTenantIdRule {
    pub message: String,
}

impl FallbackDelegationWindowRule for FallbackDelegationWindowRequiredTenantIdRule {
    fn check(&self, item: &FallbackDelegationWindow) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationWindowAudienceLengthRule {
    pub max_length: i64,
}

impl FallbackDelegationWindowRule for FallbackDelegationWindowAudienceLengthRule {
    fn check(&self, item: &FallbackDelegationWindow) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback delegation window.
#[derive(Debug, Clone)]
pub struct FallbackDelegationWindowValidator {
    pub rules: Vec<Rc<dyn FallbackDelegationWindowRule>>,
}

impl FallbackDelegationWindowValidator {
    pub fn validate(&self, item: &FallbackDelegationWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackDelegationWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDelegationWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_delegation_window_validator() -> FallbackDelegationWindowValidator {
    FallbackDelegationWindowValidator {
        rules: vec![
            Rc::new(FallbackDelegationWindowRequiredCodeRule {
                message: "fallback delegation window needs an entry".to_string(),
            }) as Rc<dyn FallbackDelegationWindowRule>,
            Rc::new(FallbackDelegationWindowAllowedLabelRule {
                allowed: vec!["weighted".to_string(), "primary".to_string()],
            }),
            Rc::new(FallbackDelegationWindowRequiredTenantIdRule {
                message: "fallback delegation window needs a value".to_string(),
            }),
            Rc::new(FallbackDelegationWindowAudienceLengthRule { max_length: 1000 }),
        ],
    }
}
