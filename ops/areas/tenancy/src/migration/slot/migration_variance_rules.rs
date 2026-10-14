use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration variance must pass. Check returns its problems, or none.
pub trait MigrationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &MigrationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationVarianceRequiredNameRule {
    pub message: String,
}

impl MigrationVarianceRule for MigrationVarianceRequiredNameRule {
    fn check(&self, item: &MigrationVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationVarianceRequiredReviewerRule {
    pub message: String,
}

impl MigrationVarianceRule for MigrationVarianceRequiredReviewerRule {
    fn check(&self, item: &MigrationVariance) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationVarianceCodePrefixRule {
    pub prefix: String,
}

impl MigrationVarianceRule for MigrationVarianceCodePrefixRule {
    fn check(&self, item: &MigrationVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationVarianceCodeLengthRule {
    pub max_length: i64,
}

impl MigrationVarianceRule for MigrationVarianceCodeLengthRule {
    fn check(&self, item: &MigrationVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationVarianceRequiredCodeRule {
    pub message: String,
}

impl MigrationVarianceRule for MigrationVarianceRequiredCodeRule {
    fn check(&self, item: &MigrationVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationVarianceAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl MigrationVarianceRule for MigrationVarianceAllowedReviewerRule {
    fn check(&self, item: &MigrationVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is missing", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration variance.
#[derive(Debug, Clone)]
pub struct MigrationVarianceValidator {
    pub rules: Vec<Rc<dyn MigrationVarianceRule>>,
}

impl MigrationVarianceValidator {
    pub fn validate(&self, item: &MigrationVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_variance_validator() -> MigrationVarianceValidator {
    MigrationVarianceValidator {
        rules: vec![
            Rc::new(MigrationVarianceRequiredNameRule {
                message: "migration variance needs an entry".to_string(),
            }) as Rc<dyn MigrationVarianceRule>,
            Rc::new(MigrationVarianceRequiredReviewerRule {
                message: "migration variance needs this field".to_string(),
            }),
            Rc::new(MigrationVarianceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(MigrationVarianceCodeLengthRule { max_length: 32 }),
            Rc::new(MigrationVarianceRequiredCodeRule {
                message: "migration variance needs this field".to_string(),
            }),
            Rc::new(MigrationVarianceAllowedReviewerRule { allowed: Vec::new() }),
        ],
    }
}
