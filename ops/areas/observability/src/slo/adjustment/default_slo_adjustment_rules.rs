use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default slo adjustment must pass. Check returns its problems, or none.
pub trait DefaultSloAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultSloAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl DefaultSloAdjustmentRule for DefaultSloAdjustmentCodePrefixRule {
    fn check(&self, item: &DefaultSloAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloAdjustmentRequiredCodeRule {
    pub message: String,
}

impl DefaultSloAdjustmentRule for DefaultSloAdjustmentRequiredCodeRule {
    fn check(&self, item: &DefaultSloAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl DefaultSloAdjustmentRule for DefaultSloAdjustmentCodeLengthRule {
    fn check(&self, item: &DefaultSloAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloAdjustmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DefaultSloAdjustmentRule for DefaultSloAdjustmentAllowedTitleRule {
    fn check(&self, item: &DefaultSloAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default slo adjustment.
#[derive(Debug, Clone)]
pub struct DefaultSloAdjustmentValidator {
    pub rules: Vec<Rc<dyn DefaultSloAdjustmentRule>>,
}

impl DefaultSloAdjustmentValidator {
    pub fn validate(&self, item: &DefaultSloAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultSloAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultSloAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_slo_adjustment_validator() -> DefaultSloAdjustmentValidator {
    DefaultSloAdjustmentValidator {
        rules: vec![
            Rc::new(DefaultSloAdjustmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn DefaultSloAdjustmentRule>,
            Rc::new(DefaultSloAdjustmentRequiredCodeRule {
                message: "default slo adjustment needs a value".to_string(),
            }),
            Rc::new(DefaultSloAdjustmentCodeLengthRule { max_length: 120 }),
            Rc::new(DefaultSloAdjustmentAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}
