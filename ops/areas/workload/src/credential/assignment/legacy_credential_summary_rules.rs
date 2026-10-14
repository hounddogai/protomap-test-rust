use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy credential summary must pass. Check returns its problems, or none.
pub trait LegacyCredentialSummaryRule: std::fmt::Debug {
    fn check(&self, item: &LegacyCredentialSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyCredentialSummaryRule for LegacyCredentialSummaryAllowedCodeRule {
    fn check(&self, item: &LegacyCredentialSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSummaryRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyCredentialSummaryRule for LegacyCredentialSummaryRevisionRangeRule {
    fn check(&self, item: &LegacyCredentialSummary) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSummaryRequiredLabelRule {
    pub message: String,
}

impl LegacyCredentialSummaryRule for LegacyCredentialSummaryRequiredLabelRule {
    fn check(&self, item: &LegacyCredentialSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSummaryCodeLengthRule {
    pub max_length: i64,
}

impl LegacyCredentialSummaryRule for LegacyCredentialSummaryCodeLengthRule {
    fn check(&self, item: &LegacyCredentialSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSummaryCodePrefixRule {
    pub prefix: String,
}

impl LegacyCredentialSummaryRule for LegacyCredentialSummaryCodePrefixRule {
    fn check(&self, item: &LegacyCredentialSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSummaryRequiredCodeRule {
    pub message: String,
}

impl LegacyCredentialSummaryRule for LegacyCredentialSummaryRequiredCodeRule {
    fn check(&self, item: &LegacyCredentialSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy credential summary.
#[derive(Debug, Clone)]
pub struct LegacyCredentialSummaryValidator {
    pub rules: Vec<Rc<dyn LegacyCredentialSummaryRule>>,
}

impl LegacyCredentialSummaryValidator {
    pub fn validate(&self, item: &LegacyCredentialSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyCredentialSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyCredentialSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_credential_summary_validator() -> LegacyCredentialSummaryValidator {
    LegacyCredentialSummaryValidator {
        rules: vec![
            Rc::new(LegacyCredentialSummaryAllowedCodeRule {
                allowed: vec!["batched".to_string(), "fallback".to_string(), "seasonal".to_string()],
            }) as Rc<dyn LegacyCredentialSummaryRule>,
            Rc::new(LegacyCredentialSummaryRevisionRangeRule { min: 120, max: 3600 }),
            Rc::new(LegacyCredentialSummaryRequiredLabelRule {
                message: "legacy credential summary needs this field".to_string(),
            }),
            Rc::new(LegacyCredentialSummaryCodeLengthRule { max_length: 32 }),
            Rc::new(LegacyCredentialSummaryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LegacyCredentialSummaryRequiredCodeRule {
                message: "legacy credential summary needs this field".to_string(),
            }),
        ],
    }
}
