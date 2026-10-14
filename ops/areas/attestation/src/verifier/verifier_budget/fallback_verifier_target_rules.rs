use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback verifier target must pass. Check returns its problems, or none.
pub trait FallbackVerifierTargetRule: std::fmt::Debug {
    fn check(&self, item: &FallbackVerifierTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierTargetNameLengthRule {
    pub max_length: i64,
}

impl FallbackVerifierTargetRule for FallbackVerifierTargetNameLengthRule {
    fn check(&self, item: &FallbackVerifierTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierTargetCodePrefixRule {
    pub prefix: String,
}

impl FallbackVerifierTargetRule for FallbackVerifierTargetCodePrefixRule {
    fn check(&self, item: &FallbackVerifierTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierTargetAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl FallbackVerifierTargetRule for FallbackVerifierTargetAllowedTimezoneRule {
    fn check(&self, item: &FallbackVerifierTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is stale", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback verifier target.
#[derive(Debug, Clone)]
pub struct FallbackVerifierTargetValidator {
    pub rules: Vec<Rc<dyn FallbackVerifierTargetRule>>,
}

impl FallbackVerifierTargetValidator {
    pub fn validate(&self, item: &FallbackVerifierTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackVerifierTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackVerifierTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_verifier_target_validator() -> FallbackVerifierTargetValidator {
    FallbackVerifierTargetValidator {
        rules: vec![
            Rc::new(FallbackVerifierTargetNameLengthRule { max_length: 16 }) as Rc<dyn FallbackVerifierTargetRule>,
            Rc::new(FallbackVerifierTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(FallbackVerifierTargetAllowedTimezoneRule {
                allowed: vec!["adaptive".to_string(), "weighted".to_string(), "tiered".to_string()],
            }),
        ],
    }
}
