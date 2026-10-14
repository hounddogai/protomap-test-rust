use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain exception must pass. Check returns its problems, or none.
pub trait TrustDomainExceptionRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainExceptionNotesLengthRule {
    pub max_length: i64,
}

impl TrustDomainExceptionRule for TrustDomainExceptionNotesLengthRule {
    fn check(&self, item: &TrustDomainException) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainExceptionRequiredTitleRule {
    pub message: String,
}

impl TrustDomainExceptionRule for TrustDomainExceptionRequiredTitleRule {
    fn check(&self, item: &TrustDomainException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainExceptionAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl TrustDomainExceptionRule for TrustDomainExceptionAllowedNotesRule {
    fn check(&self, item: &TrustDomainException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is expired", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainExceptionApprovedGuardRule {
    pub limit: i64,
}

impl TrustDomainExceptionRule for TrustDomainExceptionApprovedGuardRule {
    fn check(&self, item: &TrustDomainException) -> Vec<String> {
        if item.approved && (item.retries > self.limit) {
            return vec![format!(
                "retries {} exceeds {} while approved",
                item.retries, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainExceptionRequiredCodeRule {
    pub message: String,
}

impl TrustDomainExceptionRule for TrustDomainExceptionRequiredCodeRule {
    fn check(&self, item: &TrustDomainException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain exception.
#[derive(Debug, Clone)]
pub struct TrustDomainExceptionValidator {
    pub rules: Vec<Rc<dyn TrustDomainExceptionRule>>,
}

impl TrustDomainExceptionValidator {
    pub fn validate(&self, item: &TrustDomainException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_exception_validator() -> TrustDomainExceptionValidator {
    TrustDomainExceptionValidator {
        rules: vec![
            Rc::new(TrustDomainExceptionNotesLengthRule { max_length: 120 }) as Rc<dyn TrustDomainExceptionRule>,
            Rc::new(TrustDomainExceptionRequiredTitleRule {
                message: "trust domain exception needs an entry".to_string(),
            }),
            Rc::new(TrustDomainExceptionAllowedNotesRule { allowed: Vec::new() }),
            Rc::new(TrustDomainExceptionApprovedGuardRule { limit: 80 }),
            Rc::new(TrustDomainExceptionRequiredCodeRule {
                message: "trust domain exception needs an entry".to_string(),
            }),
        ],
    }
}
