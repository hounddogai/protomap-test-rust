use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert variance must pass. Check returns its problems, or none.
pub trait AlertVarianceRule: std::fmt::Debug {
    fn check(&self, item: &AlertVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertVarianceRequiredTitleRule {
    pub message: String,
}

impl AlertVarianceRule for AlertVarianceRequiredTitleRule {
    fn check(&self, item: &AlertVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertVarianceCodeLengthRule {
    pub max_length: i64,
}

impl AlertVarianceRule for AlertVarianceCodeLengthRule {
    fn check(&self, item: &AlertVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertVarianceContactEmailLengthRule {
    pub max_length: i64,
}

impl AlertVarianceRule for AlertVarianceContactEmailLengthRule {
    fn check(&self, item: &AlertVariance) -> Vec<String> {
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
pub struct AlertVarianceRequiredContactEmailRule {
    pub message: String,
}

impl AlertVarianceRule for AlertVarianceRequiredContactEmailRule {
    fn check(&self, item: &AlertVariance) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertVarianceCodePrefixRule {
    pub prefix: String,
}

impl AlertVarianceRule for AlertVarianceCodePrefixRule {
    fn check(&self, item: &AlertVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertVarianceAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl AlertVarianceRule for AlertVarianceAllowedContactEmailRule {
    fn check(&self, item: &AlertVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is not allowed", item.contact_email)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert variance.
#[derive(Debug, Clone)]
pub struct AlertVarianceValidator {
    pub rules: Vec<Rc<dyn AlertVarianceRule>>,
}

impl AlertVarianceValidator {
    pub fn validate(&self, item: &AlertVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_variance_validator() -> AlertVarianceValidator {
    AlertVarianceValidator {
        rules: vec![
            Rc::new(AlertVarianceRequiredTitleRule {
                message: "alert variance needs this field".to_string(),
            }) as Rc<dyn AlertVarianceRule>,
            Rc::new(AlertVarianceCodeLengthRule { max_length: 64 }),
            Rc::new(AlertVarianceContactEmailLengthRule { max_length: 32 }),
            Rc::new(AlertVarianceRequiredContactEmailRule {
                message: "alert variance needs this field".to_string(),
            }),
            Rc::new(AlertVarianceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(AlertVarianceAllowedContactEmailRule {
                allowed: vec!["capped".to_string(), "capped".to_string(), "weighted".to_string()],
            }),
        ],
    }
}
