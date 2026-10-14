use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation exception must pass. Check returns its problems, or none.
pub trait DelegationExceptionRule: std::fmt::Debug {
    fn check(&self, item: &DelegationException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationExceptionRequiredTitleRule {
    pub message: String,
}

impl DelegationExceptionRule for DelegationExceptionRequiredTitleRule {
    fn check(&self, item: &DelegationException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationExceptionRequiredCodeRule {
    pub message: String,
}

impl DelegationExceptionRule for DelegationExceptionRequiredCodeRule {
    fn check(&self, item: &DelegationException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationExceptionNamespaceLengthRule {
    pub max_length: i64,
}

impl DelegationExceptionRule for DelegationExceptionNamespaceLengthRule {
    fn check(&self, item: &DelegationException) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationExceptionAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl DelegationExceptionRule for DelegationExceptionAllowedNamespaceRule {
    fn check(&self, item: &DelegationException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is not allowed", item.namespace)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation exception.
#[derive(Debug, Clone)]
pub struct DelegationExceptionValidator {
    pub rules: Vec<Rc<dyn DelegationExceptionRule>>,
}

impl DelegationExceptionValidator {
    pub fn validate(&self, item: &DelegationException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_exception_validator() -> DelegationExceptionValidator {
    DelegationExceptionValidator {
        rules: vec![
            Rc::new(DelegationExceptionRequiredTitleRule {
                message: "delegation exception needs this field".to_string(),
            }) as Rc<dyn DelegationExceptionRule>,
            Rc::new(DelegationExceptionRequiredCodeRule {
                message: "delegation exception needs a value".to_string(),
            }),
            Rc::new(DelegationExceptionNamespaceLengthRule { max_length: 1000 }),
            Rc::new(DelegationExceptionAllowedNamespaceRule {
                allowed: vec!["pending".to_string()],
            }),
        ],
    }
}
