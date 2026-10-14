use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote manifest must pass. Check returns its problems, or none.
pub trait QuoteManifestRule: std::fmt::Debug {
    fn check(&self, item: &QuoteManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteManifestScopeLengthRule {
    pub max_length: i64,
}

impl QuoteManifestRule for QuoteManifestScopeLengthRule {
    fn check(&self, item: &QuoteManifest) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteManifestAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl QuoteManifestRule for QuoteManifestAllowedScopeRule {
    fn check(&self, item: &QuoteManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is expired", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteManifestTitleLengthRule {
    pub max_length: i64,
}

impl QuoteManifestRule for QuoteManifestTitleLengthRule {
    fn check(&self, item: &QuoteManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteManifestNameLengthRule {
    pub max_length: i64,
}

impl QuoteManifestRule for QuoteManifestNameLengthRule {
    fn check(&self, item: &QuoteManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteManifestRequiredNameRule {
    pub message: String,
}

impl QuoteManifestRule for QuoteManifestRequiredNameRule {
    fn check(&self, item: &QuoteManifest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteManifestRequiredTitleRule {
    pub message: String,
}

impl QuoteManifestRule for QuoteManifestRequiredTitleRule {
    fn check(&self, item: &QuoteManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote manifest.
#[derive(Debug, Clone)]
pub struct QuoteManifestValidator {
    pub rules: Vec<Rc<dyn QuoteManifestRule>>,
}

impl QuoteManifestValidator {
    pub fn validate(&self, item: &QuoteManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_manifest_validator() -> QuoteManifestValidator {
    QuoteManifestValidator {
        rules: vec![
            Rc::new(QuoteManifestScopeLengthRule { max_length: 1000 }) as Rc<dyn QuoteManifestRule>,
            Rc::new(QuoteManifestAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(QuoteManifestTitleLengthRule { max_length: 80 }),
            Rc::new(QuoteManifestNameLengthRule { max_length: 1000 }),
            Rc::new(QuoteManifestRequiredNameRule {
                message: "quote manifest needs a value".to_string(),
            }),
            Rc::new(QuoteManifestRequiredTitleRule {
                message: "quote manifest needs a value".to_string(),
            }),
        ],
    }
}
