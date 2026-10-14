use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default action item adjustment must pass. Check returns its problems, or none.
pub trait DefaultActionItemAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultActionItemAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAdjustmentRequiredScopeRule {
    pub message: String,
}

impl DefaultActionItemAdjustmentRule for DefaultActionItemAdjustmentRequiredScopeRule {
    fn check(&self, item: &DefaultActionItemAdjustment) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl DefaultActionItemAdjustmentRule for DefaultActionItemAdjustmentCodeLengthRule {
    fn check(&self, item: &DefaultActionItemAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAdjustmentRequiredCodeRule {
    pub message: String,
}

impl DefaultActionItemAdjustmentRule for DefaultActionItemAdjustmentRequiredCodeRule {
    fn check(&self, item: &DefaultActionItemAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAdjustmentApprovedGuardRule {
    pub limit: f64,
}

impl DefaultActionItemAdjustmentRule for DefaultActionItemAdjustmentApprovedGuardRule {
    fn check(&self, item: &DefaultActionItemAdjustment) -> Vec<String> {
        if item.approved && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while approved",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default action item adjustment.
#[derive(Debug, Clone)]
pub struct DefaultActionItemAdjustmentValidatorExtra {
    pub rules: Vec<Rc<dyn DefaultActionItemAdjustmentRule>>,
}

impl DefaultActionItemAdjustmentValidatorExtra {
    pub fn validate(&self, item: &DefaultActionItemAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultActionItemAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultActionItemAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_action_item_adjustment_validator() -> DefaultActionItemAdjustmentValidatorExtra {
    DefaultActionItemAdjustmentValidatorExtra {
        rules: vec![
            Rc::new(DefaultActionItemAdjustmentRequiredScopeRule {
                message: "default action item adjustment needs this field".to_string(),
            }) as Rc<dyn DefaultActionItemAdjustmentRule>,
            Rc::new(DefaultActionItemAdjustmentCodeLengthRule { max_length: 80 }),
            Rc::new(DefaultActionItemAdjustmentRequiredCodeRule {
                message: "default action item adjustment needs an entry".to_string(),
            }),
            Rc::new(DefaultActionItemAdjustmentApprovedGuardRule { limit: 3.5 }),
        ],
    }
}
