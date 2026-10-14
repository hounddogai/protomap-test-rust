use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation claim must pass. Check returns its problems, or none.
pub trait IsolationClaimRule: std::fmt::Debug {
    fn check(&self, item: &IsolationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationClaimAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl IsolationClaimRule for IsolationClaimAllowedTitleRule {
    fn check(&self, item: &IsolationClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationClaimRequiredScopeRule {
    pub message: String,
}

impl IsolationClaimRule for IsolationClaimRequiredScopeRule {
    fn check(&self, item: &IsolationClaim) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationClaimCodePrefixRule {
    pub prefix: String,
}

impl IsolationClaimRule for IsolationClaimCodePrefixRule {
    fn check(&self, item: &IsolationClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationClaimRequiredTitleRule {
    pub message: String,
}

impl IsolationClaimRule for IsolationClaimRequiredTitleRule {
    fn check(&self, item: &IsolationClaim) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationClaimRequiredCipherRule {
    pub message: String,
}

impl IsolationClaimRule for IsolationClaimRequiredCipherRule {
    fn check(&self, item: &IsolationClaim) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation claim.
#[derive(Debug, Clone)]
pub struct IsolationClaimValidator {
    pub rules: Vec<Rc<dyn IsolationClaimRule>>,
}

impl IsolationClaimValidator {
    pub fn validate(&self, item: &IsolationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_claim_validator() -> IsolationClaimValidator {
    IsolationClaimValidator {
        rules: vec![
            Rc::new(IsolationClaimAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn IsolationClaimRule>,
            Rc::new(IsolationClaimRequiredScopeRule {
                message: "isolation claim needs an entry".to_string(),
            }),
            Rc::new(IsolationClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(IsolationClaimRequiredTitleRule {
                message: "isolation claim needs a value".to_string(),
            }),
            Rc::new(IsolationClaimRequiredCipherRule {
                message: "isolation claim needs this field".to_string(),
            }),
        ],
    }
}
