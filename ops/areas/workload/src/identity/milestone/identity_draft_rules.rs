use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity draft must pass. Check returns its problems, or none.
pub trait IdentityDraftRule: std::fmt::Debug {
    fn check(&self, item: &IdentityDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDraftMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IdentityDraftRule for IdentityDraftMinimumRangeRule {
    fn check(&self, item: &IdentityDraft) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl IdentityDraftRule for IdentityDraftAllowedCodeRule {
    fn check(&self, item: &IdentityDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDraftTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityDraftRule for IdentityDraftTtlSecondsRangeRule {
    fn check(&self, item: &IdentityDraft) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDraftContactEmailLengthRule {
    pub max_length: i64,
}

impl IdentityDraftRule for IdentityDraftContactEmailLengthRule {
    fn check(&self, item: &IdentityDraft) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity draft.
#[derive(Debug, Clone)]
pub struct IdentityDraftValidator {
    pub rules: Vec<Rc<dyn IdentityDraftRule>>,
}

impl IdentityDraftValidator {
    pub fn validate(&self, item: &IdentityDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_draft_validator() -> IdentityDraftValidator {
    IdentityDraftValidator {
        rules: vec![
            Rc::new(IdentityDraftMinimumRangeRule { min: 10.0, max: 1772.0 }) as Rc<dyn IdentityDraftRule>,
            Rc::new(IdentityDraftAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(IdentityDraftTtlSecondsRangeRule { min: 64, max: 120 }),
            Rc::new(IdentityDraftContactEmailLengthRule { max_length: 86400 }),
        ],
    }
}
