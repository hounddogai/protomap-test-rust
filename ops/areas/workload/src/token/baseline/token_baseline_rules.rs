use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token baseline must pass. Check returns its problems, or none.
pub trait TokenBaselineRule: std::fmt::Debug {
    fn check(&self, item: &TokenBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBaselineCodeLengthRule {
    pub max_length: i64,
}

impl TokenBaselineRule for TokenBaselineCodeLengthRule {
    fn check(&self, item: &TokenBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBaselineRequiredCodeRule {
    pub message: String,
}

impl TokenBaselineRule for TokenBaselineRequiredCodeRule {
    fn check(&self, item: &TokenBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBaselineRequiredLabelRule {
    pub message: String,
}

impl TokenBaselineRule for TokenBaselineRequiredLabelRule {
    fn check(&self, item: &TokenBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBaselineAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TokenBaselineRule for TokenBaselineAllowedCodeRule {
    fn check(&self, item: &TokenBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBaselineCodePrefixRule {
    pub prefix: String,
}

impl TokenBaselineRule for TokenBaselineCodePrefixRule {
    fn check(&self, item: &TokenBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token baseline.
#[derive(Debug, Clone)]
pub struct TokenBaselineValidator {
    pub rules: Vec<Rc<dyn TokenBaselineRule>>,
}

impl TokenBaselineValidator {
    pub fn validate(&self, item: &TokenBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_baseline_validator() -> TokenBaselineValidator {
    TokenBaselineValidator {
        rules: vec![
            Rc::new(TokenBaselineCodeLengthRule { max_length: 86400 }) as Rc<dyn TokenBaselineRule>,
            Rc::new(TokenBaselineRequiredCodeRule {
                message: "token baseline needs a value".to_string(),
            }),
            Rc::new(TokenBaselineRequiredLabelRule {
                message: "token baseline needs this field".to_string(),
            }),
            Rc::new(TokenBaselineAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "adaptive".to_string()],
            }),
            Rc::new(TokenBaselineCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
