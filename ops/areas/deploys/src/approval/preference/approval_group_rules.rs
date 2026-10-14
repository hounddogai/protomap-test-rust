use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval group must pass. Check returns its problems, or none.
pub trait ApprovalGroupRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalGroupRequiredCodeRule {
    pub message: String,
}

impl ApprovalGroupRule for ApprovalGroupRequiredCodeRule {
    fn check(&self, item: &ApprovalGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalGroupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ApprovalGroupRule for ApprovalGroupAllowedCodeRule {
    fn check(&self, item: &ApprovalGroup) -> Vec<String> {
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
pub struct ApprovalGroupCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalGroupRule for ApprovalGroupCeilingRangeRule {
    fn check(&self, item: &ApprovalGroup) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalGroupCipherLengthRule {
    pub max_length: i64,
}

impl ApprovalGroupRule for ApprovalGroupCipherLengthRule {
    fn check(&self, item: &ApprovalGroup) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalGroupRequiredCipherRule {
    pub message: String,
}

impl ApprovalGroupRule for ApprovalGroupRequiredCipherRule {
    fn check(&self, item: &ApprovalGroup) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval group.
#[derive(Debug, Clone)]
pub struct ApprovalGroupValidator {
    pub rules: Vec<Rc<dyn ApprovalGroupRule>>,
}

impl ApprovalGroupValidator {
    pub fn validate(&self, item: &ApprovalGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_group_validator() -> ApprovalGroupValidator {
    ApprovalGroupValidator {
        rules: vec![
            Rc::new(ApprovalGroupRequiredCodeRule {
                message: "approval group needs this field".to_string(),
            }) as Rc<dyn ApprovalGroupRule>,
            Rc::new(ApprovalGroupAllowedCodeRule {
                allowed: vec!["lenient".to_string()],
            }),
            Rc::new(ApprovalGroupCeilingRangeRule { min: 5.3, max: 1550.0 }),
            Rc::new(ApprovalGroupCipherLengthRule { max_length: 1000 }),
            Rc::new(ApprovalGroupRequiredCipherRule {
                message: "approval group needs a value".to_string(),
            }),
        ],
    }
}
