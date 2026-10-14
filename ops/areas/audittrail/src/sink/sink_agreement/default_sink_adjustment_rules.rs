use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default sink adjustment must pass. Check returns its problems, or none.
pub trait DefaultSinkAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultSinkAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkAdjustmentUnitLengthRule {
    pub max_length: i64,
}

impl DefaultSinkAdjustmentRule for DefaultSinkAdjustmentUnitLengthRule {
    fn check(&self, item: &DefaultSinkAdjustment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkAdjustmentRequiredCodeRule {
    pub message: String,
}

impl DefaultSinkAdjustmentRule for DefaultSinkAdjustmentRequiredCodeRule {
    fn check(&self, item: &DefaultSinkAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl DefaultSinkAdjustmentRule for DefaultSinkAdjustmentCodeLengthRule {
    fn check(&self, item: &DefaultSinkAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl DefaultSinkAdjustmentRule for DefaultSinkAdjustmentCodePrefixRule {
    fn check(&self, item: &DefaultSinkAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkAdjustmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl DefaultSinkAdjustmentRule for DefaultSinkAdjustmentOwnerIdLengthRule {
    fn check(&self, item: &DefaultSinkAdjustment) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default sink adjustment.
#[derive(Debug, Clone)]
pub struct DefaultSinkAdjustmentValidator {
    pub rules: Vec<Rc<dyn DefaultSinkAdjustmentRule>>,
}

impl DefaultSinkAdjustmentValidator {
    pub fn validate(&self, item: &DefaultSinkAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultSinkAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultSinkAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_sink_adjustment_validator() -> DefaultSinkAdjustmentValidator {
    DefaultSinkAdjustmentValidator {
        rules: vec![
            Rc::new(DefaultSinkAdjustmentUnitLengthRule { max_length: 16 }) as Rc<dyn DefaultSinkAdjustmentRule>,
            Rc::new(DefaultSinkAdjustmentRequiredCodeRule {
                message: "default sink adjustment needs an entry".to_string(),
            }),
            Rc::new(DefaultSinkAdjustmentCodeLengthRule { max_length: 86400 }),
            Rc::new(DefaultSinkAdjustmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultSinkAdjustmentOwnerIdLengthRule { max_length: 16 }),
        ],
    }
}
