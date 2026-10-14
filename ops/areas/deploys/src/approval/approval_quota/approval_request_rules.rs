use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval request must pass. Check returns its problems, or none.
pub trait ApprovalRequestRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRequestAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl ApprovalRequestRule for ApprovalRequestAllowedCurrencyRule {
    fn check(&self, item: &ApprovalRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is below minimum", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRequestRequiredCurrencyRule {
    pub message: String,
}

impl ApprovalRequestRule for ApprovalRequestRequiredCurrencyRule {
    fn check(&self, item: &ApprovalRequest) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRequestCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalRequestRule for ApprovalRequestCodeLengthRule {
    fn check(&self, item: &ApprovalRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval request.
#[derive(Debug, Clone)]
pub struct ApprovalRequestValidator {
    pub rules: Vec<Rc<dyn ApprovalRequestRule>>,
}

impl ApprovalRequestValidator {
    pub fn validate(&self, item: &ApprovalRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_request_validator() -> ApprovalRequestValidator {
    ApprovalRequestValidator {
        rules: vec![
            Rc::new(ApprovalRequestAllowedCurrencyRule {
                allowed: vec!["seasonal".to_string(), "primary".to_string()],
            }) as Rc<dyn ApprovalRequestRule>,
            Rc::new(ApprovalRequestRequiredCurrencyRule {
                message: "approval request needs this field".to_string(),
            }),
            Rc::new(ApprovalRequestCodeLengthRule { max_length: 120 }),
        ],
    }
}
