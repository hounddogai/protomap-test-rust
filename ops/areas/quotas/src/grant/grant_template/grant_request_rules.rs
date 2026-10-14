use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant request must pass. Check returns its problems, or none.
pub trait GrantRequestRule: std::fmt::Debug {
    fn check(&self, item: &GrantRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRequestIssuerLengthRule {
    pub max_length: i64,
}

impl GrantRequestRule for GrantRequestIssuerLengthRule {
    fn check(&self, item: &GrantRequest) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRequestCodeLengthRule {
    pub max_length: i64,
}

impl GrantRequestRule for GrantRequestCodeLengthRule {
    fn check(&self, item: &GrantRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRequestLabelLengthRule {
    pub max_length: i64,
}

impl GrantRequestRule for GrantRequestLabelLengthRule {
    fn check(&self, item: &GrantRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantRequestRequiredCurrencyRule {
    pub message: String,
}

impl GrantRequestRule for GrantRequestRequiredCurrencyRule {
    fn check(&self, item: &GrantRequest) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant request.
#[derive(Debug, Clone)]
pub struct GrantRequestValidator {
    pub rules: Vec<Rc<dyn GrantRequestRule>>,
}

impl GrantRequestValidator {
    pub fn validate(&self, item: &GrantRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_request_validator() -> GrantRequestValidator {
    GrantRequestValidator {
        rules: vec![
            Rc::new(GrantRequestIssuerLengthRule { max_length: 1000 }) as Rc<dyn GrantRequestRule>,
            Rc::new(GrantRequestCodeLengthRule { max_length: 120 }),
            Rc::new(GrantRequestLabelLengthRule { max_length: 255 }),
            Rc::new(GrantRequestRequiredCurrencyRule {
                message: "grant request needs a value".to_string(),
            }),
        ],
    }
}
