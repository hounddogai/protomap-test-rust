use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback exception must pass. Check returns its problems, or none.
pub trait RollbackExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RollbackException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackExceptionCodePrefixRule {
    pub prefix: String,
}

impl RollbackExceptionRule for RollbackExceptionCodePrefixRule {
    fn check(&self, item: &RollbackException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackExceptionAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl RollbackExceptionRule for RollbackExceptionAllowedCipherRule {
    fn check(&self, item: &RollbackException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is missing", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackExceptionRequiredCipherRule {
    pub message: String,
}

impl RollbackExceptionRule for RollbackExceptionRequiredCipherRule {
    fn check(&self, item: &RollbackException) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback exception.
#[derive(Debug, Clone)]
pub struct RollbackExceptionValidator {
    pub rules: Vec<Rc<dyn RollbackExceptionRule>>,
}

impl RollbackExceptionValidator {
    pub fn validate(&self, item: &RollbackException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_exception_validator() -> RollbackExceptionValidator {
    RollbackExceptionValidator {
        rules: vec![
            Rc::new(RollbackExceptionCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn RollbackExceptionRule>,
            Rc::new(RollbackExceptionAllowedCipherRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(RollbackExceptionRequiredCipherRule {
                message: "rollback exception needs a value".to_string(),
            }),
        ],
    }
}
