use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention mapping must pass. Check returns its problems, or none.
pub trait RetentionMappingRule: std::fmt::Debug {
    fn check(&self, item: &RetentionMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMappingCodePrefixRule {
    pub prefix: String,
}

impl RetentionMappingRule for RetentionMappingCodePrefixRule {
    fn check(&self, item: &RetentionMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMappingRequiredTitleRule {
    pub message: String,
}

impl RetentionMappingRule for RetentionMappingRequiredTitleRule {
    fn check(&self, item: &RetentionMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMappingAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl RetentionMappingRule for RetentionMappingAllowedScopeRule {
    fn check(&self, item: &RetentionMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is inconsistent", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMappingNameLengthRule {
    pub max_length: i64,
}

impl RetentionMappingRule for RetentionMappingNameLengthRule {
    fn check(&self, item: &RetentionMapping) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention mapping.
#[derive(Debug, Clone)]
pub struct RetentionMappingValidator {
    pub rules: Vec<Rc<dyn RetentionMappingRule>>,
}

impl RetentionMappingValidator {
    pub fn validate(&self, item: &RetentionMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_mapping_validator() -> RetentionMappingValidator {
    RetentionMappingValidator {
        rules: vec![
            Rc::new(RetentionMappingCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn RetentionMappingRule>,
            Rc::new(RetentionMappingRequiredTitleRule {
                message: "retention mapping needs a value".to_string(),
            }),
            Rc::new(RetentionMappingAllowedScopeRule {
                allowed: vec!["capped".to_string(), "seasonal".to_string(), "default".to_string()],
            }),
            Rc::new(RetentionMappingNameLengthRule { max_length: 80 }),
        ],
    }
}
