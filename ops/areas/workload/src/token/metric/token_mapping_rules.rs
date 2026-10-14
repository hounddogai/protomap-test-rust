use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token mapping must pass. Check returns its problems, or none.
pub trait TokenMappingRule: std::fmt::Debug {
    fn check(&self, item: &TokenMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMappingCodePrefixRule {
    pub prefix: String,
}

impl TokenMappingRule for TokenMappingCodePrefixRule {
    fn check(&self, item: &TokenMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMappingSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenMappingRule for TokenMappingSequenceRangeRule {
    fn check(&self, item: &TokenMapping) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMappingAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TokenMappingRule for TokenMappingAllowedTitleRule {
    fn check(&self, item: &TokenMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMappingScopeLengthRule {
    pub max_length: i64,
}

impl TokenMappingRule for TokenMappingScopeLengthRule {
    fn check(&self, item: &TokenMapping) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMappingApprovedGuardRule {
    pub limit: i64,
}

impl TokenMappingRule for TokenMappingApprovedGuardRule {
    fn check(&self, item: &TokenMapping) -> Vec<String> {
        if item.approved && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while approved",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a token mapping.
#[derive(Debug, Clone)]
pub struct TokenMappingValidator {
    pub rules: Vec<Rc<dyn TokenMappingRule>>,
}

impl TokenMappingValidator {
    pub fn validate(&self, item: &TokenMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_mapping_validator() -> TokenMappingValidator {
    TokenMappingValidator {
        rules: vec![
            Rc::new(TokenMappingCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn TokenMappingRule>,
            Rc::new(TokenMappingSequenceRangeRule { min: 80, max: 255 }),
            Rc::new(TokenMappingAllowedTitleRule {
                allowed: vec!["primary".to_string(), "tiered".to_string()],
            }),
            Rc::new(TokenMappingScopeLengthRule { max_length: 32 }),
            Rc::new(TokenMappingApprovedGuardRule { limit: 3600 }),
        ],
    }
}
