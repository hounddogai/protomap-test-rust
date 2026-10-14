use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched identity target must pass. Check returns its problems, or none.
pub trait BatchedIdentityTargetRule: std::fmt::Debug {
    fn check(&self, item: &BatchedIdentityTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityTargetRequiredCodeRule {
    pub message: String,
}

impl BatchedIdentityTargetRule for BatchedIdentityTargetRequiredCodeRule {
    fn check(&self, item: &BatchedIdentityTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityTargetAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl BatchedIdentityTargetRule for BatchedIdentityTargetAllowedCategoryRule {
    fn check(&self, item: &BatchedIdentityTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is out of range", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityTargetCodePrefixRule {
    pub prefix: String,
}

impl BatchedIdentityTargetRule for BatchedIdentityTargetCodePrefixRule {
    fn check(&self, item: &BatchedIdentityTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityTargetRequiredNameRule {
    pub message: String,
}

impl BatchedIdentityTargetRule for BatchedIdentityTargetRequiredNameRule {
    fn check(&self, item: &BatchedIdentityTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityTargetCategoryLengthRule {
    pub max_length: i64,
}

impl BatchedIdentityTargetRule for BatchedIdentityTargetCategoryLengthRule {
    fn check(&self, item: &BatchedIdentityTarget) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched identity target.
#[derive(Debug, Clone)]
pub struct BatchedIdentityTargetValidator {
    pub rules: Vec<Rc<dyn BatchedIdentityTargetRule>>,
}

impl BatchedIdentityTargetValidator {
    pub fn validate(&self, item: &BatchedIdentityTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedIdentityTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedIdentityTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_identity_target_validator() -> BatchedIdentityTargetValidator {
    BatchedIdentityTargetValidator {
        rules: vec![
            Rc::new(BatchedIdentityTargetRequiredCodeRule {
                message: "batched identity target needs an entry".to_string(),
            }) as Rc<dyn BatchedIdentityTargetRule>,
            Rc::new(BatchedIdentityTargetAllowedCategoryRule {
                allowed: vec!["adaptive".to_string(), "lenient".to_string(), "seasonal".to_string()],
            }),
            Rc::new(BatchedIdentityTargetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BatchedIdentityTargetRequiredNameRule {
                message: "batched identity target needs this field".to_string(),
            }),
            Rc::new(BatchedIdentityTargetCategoryLengthRule { max_length: 32 }),
        ],
    }
}
