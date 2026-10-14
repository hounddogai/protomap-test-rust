use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration threshold must pass. Check returns its problems, or none.
pub trait MigrationThresholdRule: std::fmt::Debug {
    fn check(&self, item: &MigrationThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationThresholdScopeLengthRule {
    pub max_length: i64,
}

impl MigrationThresholdRule for MigrationThresholdScopeLengthRule {
    fn check(&self, item: &MigrationThreshold) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationThresholdCodeLengthRule {
    pub max_length: i64,
}

impl MigrationThresholdRule for MigrationThresholdCodeLengthRule {
    fn check(&self, item: &MigrationThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationThresholdAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl MigrationThresholdRule for MigrationThresholdAllowedScopeRule {
    fn check(&self, item: &MigrationThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is stale", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationThresholdRequiredScopeRule {
    pub message: String,
}

impl MigrationThresholdRule for MigrationThresholdRequiredScopeRule {
    fn check(&self, item: &MigrationThreshold) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationThresholdRequiredLabelRule {
    pub message: String,
}

impl MigrationThresholdRule for MigrationThresholdRequiredLabelRule {
    fn check(&self, item: &MigrationThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationThresholdCodePrefixRule {
    pub prefix: String,
}

impl MigrationThresholdRule for MigrationThresholdCodePrefixRule {
    fn check(&self, item: &MigrationThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration threshold.
#[derive(Debug, Clone)]
pub struct MigrationThresholdValidator {
    pub rules: Vec<Rc<dyn MigrationThresholdRule>>,
}

impl MigrationThresholdValidator {
    pub fn validate(&self, item: &MigrationThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_threshold_validator() -> MigrationThresholdValidator {
    MigrationThresholdValidator {
        rules: vec![
            Rc::new(MigrationThresholdScopeLengthRule { max_length: 16 }) as Rc<dyn MigrationThresholdRule>,
            Rc::new(MigrationThresholdCodeLengthRule { max_length: 120 }),
            Rc::new(MigrationThresholdAllowedScopeRule {
                allowed: vec!["strict".to_string(), "capped".to_string()],
            }),
            Rc::new(MigrationThresholdRequiredScopeRule {
                message: "migration threshold needs an entry".to_string(),
            }),
            Rc::new(MigrationThresholdRequiredLabelRule {
                message: "migration threshold needs this field".to_string(),
            }),
            Rc::new(MigrationThresholdCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
