use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction manifest must pass. Check returns its problems, or none.
pub trait RedactionManifestRule: std::fmt::Debug {
    fn check(&self, item: &RedactionManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionManifestRequiredTitleRule {
    pub message: String,
}

impl RedactionManifestRule for RedactionManifestRequiredTitleRule {
    fn check(&self, item: &RedactionManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionManifestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionManifestRule for RedactionManifestTtlSecondsRangeRule {
    fn check(&self, item: &RedactionManifest) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionManifestCodePrefixRule {
    pub prefix: String,
}

impl RedactionManifestRule for RedactionManifestCodePrefixRule {
    fn check(&self, item: &RedactionManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionManifestTitleLengthRule {
    pub max_length: i64,
}

impl RedactionManifestRule for RedactionManifestTitleLengthRule {
    fn check(&self, item: &RedactionManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionManifestCodeLengthRule {
    pub max_length: i64,
}

impl RedactionManifestRule for RedactionManifestCodeLengthRule {
    fn check(&self, item: &RedactionManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionManifestRequiredCodeRule {
    pub message: String,
}

impl RedactionManifestRule for RedactionManifestRequiredCodeRule {
    fn check(&self, item: &RedactionManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction manifest.
#[derive(Debug, Clone)]
pub struct RedactionManifestValidator {
    pub rules: Vec<Rc<dyn RedactionManifestRule>>,
}

impl RedactionManifestValidator {
    pub fn validate(&self, item: &RedactionManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_manifest_validator() -> RedactionManifestValidator {
    RedactionManifestValidator {
        rules: vec![
            Rc::new(RedactionManifestRequiredTitleRule {
                message: "redaction manifest needs this field".to_string(),
            }) as Rc<dyn RedactionManifestRule>,
            Rc::new(RedactionManifestTtlSecondsRangeRule { min: 120, max: 255 }),
            Rc::new(RedactionManifestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RedactionManifestTitleLengthRule { max_length: 64 }),
            Rc::new(RedactionManifestCodeLengthRule { max_length: 255 }),
            Rc::new(RedactionManifestRequiredCodeRule {
                message: "redaction manifest needs this field".to_string(),
            }),
        ],
    }
}
