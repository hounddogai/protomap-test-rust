use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding window must pass. Check returns its problems, or none.
pub trait BindingWindowRule: std::fmt::Debug {
    fn check(&self, item: &BindingWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindowAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BindingWindowRule for BindingWindowAllowedCodeRule {
    fn check(&self, item: &BindingWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindowRequiredCodeRule {
    pub message: String,
}

impl BindingWindowRule for BindingWindowRequiredCodeRule {
    fn check(&self, item: &BindingWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindowCodeLengthRule {
    pub max_length: i64,
}

impl BindingWindowRule for BindingWindowCodeLengthRule {
    fn check(&self, item: &BindingWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindowTitleLengthRule {
    pub max_length: i64,
}

impl BindingWindowRule for BindingWindowTitleLengthRule {
    fn check(&self, item: &BindingWindow) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindowContactEmailLengthRule {
    pub max_length: i64,
}

impl BindingWindowRule for BindingWindowContactEmailLengthRule {
    fn check(&self, item: &BindingWindow) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindowRequiredPrincipalRule {
    pub message: String,
}

impl BindingWindowRule for BindingWindowRequiredPrincipalRule {
    fn check(&self, item: &BindingWindow) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding window.
#[derive(Debug, Clone)]
pub struct BindingWindowValidator {
    pub rules: Vec<Rc<dyn BindingWindowRule>>,
}

impl BindingWindowValidator {
    pub fn validate(&self, item: &BindingWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_window_validator() -> BindingWindowValidator {
    BindingWindowValidator {
        rules: vec![
            Rc::new(BindingWindowAllowedCodeRule {
                allowed: vec!["global".to_string()],
            }) as Rc<dyn BindingWindowRule>,
            Rc::new(BindingWindowRequiredCodeRule {
                message: "binding window needs a value".to_string(),
            }),
            Rc::new(BindingWindowCodeLengthRule { max_length: 1000 }),
            Rc::new(BindingWindowTitleLengthRule { max_length: 80 }),
            Rc::new(BindingWindowContactEmailLengthRule { max_length: 16 }),
            Rc::new(BindingWindowRequiredPrincipalRule {
                message: "binding window needs this field".to_string(),
            }),
        ],
    }
}
