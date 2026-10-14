use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote entry must pass. Check returns its problems, or none.
pub trait QuoteEntryRule: std::fmt::Debug {
    fn check(&self, item: &QuoteEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEntryCodePrefixRule {
    pub prefix: String,
}

impl QuoteEntryRule for QuoteEntryCodePrefixRule {
    fn check(&self, item: &QuoteEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEntryRequiredCodeRule {
    pub message: String,
}

impl QuoteEntryRule for QuoteEntryRequiredCodeRule {
    fn check(&self, item: &QuoteEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEntryRequiredNameRule {
    pub message: String,
}

impl QuoteEntryRule for QuoteEntryRequiredNameRule {
    fn check(&self, item: &QuoteEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEntryRegionLengthRule {
    pub max_length: i64,
}

impl QuoteEntryRule for QuoteEntryRegionLengthRule {
    fn check(&self, item: &QuoteEntry) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl QuoteEntryRule for QuoteEntryAllowedCodeRule {
    fn check(&self, item: &QuoteEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote entry.
#[derive(Debug, Clone)]
pub struct QuoteEntryValidator {
    pub rules: Vec<Rc<dyn QuoteEntryRule>>,
}

impl QuoteEntryValidator {
    pub fn validate(&self, item: &QuoteEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_entry_validator() -> QuoteEntryValidator {
    QuoteEntryValidator {
        rules: vec![
            Rc::new(QuoteEntryCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn QuoteEntryRule>,
            Rc::new(QuoteEntryRequiredCodeRule {
                message: "quote entry needs an entry".to_string(),
            }),
            Rc::new(QuoteEntryRequiredNameRule {
                message: "quote entry needs this field".to_string(),
            }),
            Rc::new(QuoteEntryRegionLengthRule { max_length: 255 }),
            Rc::new(QuoteEntryAllowedCodeRule {
                allowed: vec!["global".to_string(), "fallback".to_string(), "default".to_string()],
            }),
        ],
    }
}
