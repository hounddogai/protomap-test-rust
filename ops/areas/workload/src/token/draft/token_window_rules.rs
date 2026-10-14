use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token window must pass. Check returns its problems, or none.
pub trait TokenWindowRule: std::fmt::Debug {
    fn check(&self, item: &TokenWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowRequiredCodeRule {
    pub message: String,
}

impl TokenWindowRule for TokenWindowRequiredCodeRule {
    fn check(&self, item: &TokenWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowRequiredLocaleRule {
    pub message: String,
}

impl TokenWindowRule for TokenWindowRequiredLocaleRule {
    fn check(&self, item: &TokenWindow) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowRequiredLabelRule {
    pub message: String,
}

impl TokenWindowRule for TokenWindowRequiredLabelRule {
    fn check(&self, item: &TokenWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowCodePrefixRule {
    pub prefix: String,
}

impl TokenWindowRule for TokenWindowCodePrefixRule {
    fn check(&self, item: &TokenWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowLocaleLengthRule {
    pub max_length: i64,
}

impl TokenWindowRule for TokenWindowLocaleLengthRule {
    fn check(&self, item: &TokenWindow) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowLabelLengthRule {
    pub max_length: i64,
}

impl TokenWindowRule for TokenWindowLabelLengthRule {
    fn check(&self, item: &TokenWindow) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token window.
#[derive(Debug, Clone)]
pub struct TokenWindowValidator {
    pub rules: Vec<Rc<dyn TokenWindowRule>>,
}

impl TokenWindowValidator {
    pub fn validate(&self, item: &TokenWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_window_validator() -> TokenWindowValidator {
    TokenWindowValidator {
        rules: vec![
            Rc::new(TokenWindowRequiredCodeRule {
                message: "token window needs a value".to_string(),
            }) as Rc<dyn TokenWindowRule>,
            Rc::new(TokenWindowRequiredLocaleRule {
                message: "token window needs a value".to_string(),
            }),
            Rc::new(TokenWindowRequiredLabelRule {
                message: "token window needs this field".to_string(),
            }),
            Rc::new(TokenWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TokenWindowLocaleLengthRule { max_length: 32 }),
            Rc::new(TokenWindowLabelLengthRule { max_length: 120 }),
        ],
    }
}
