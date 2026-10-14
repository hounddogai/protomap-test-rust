use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem slot must pass. Check returns its problems, or none.
pub trait PostmortemSlotRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSlotRequiredSourceRule {
    pub message: String,
}

impl PostmortemSlotRule for PostmortemSlotRequiredSourceRule {
    fn check(&self, item: &PostmortemSlot) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSlotCodePrefixRule {
    pub prefix: String,
}

impl PostmortemSlotRule for PostmortemSlotCodePrefixRule {
    fn check(&self, item: &PostmortemSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSlotNamespaceLengthRule {
    pub max_length: i64,
}

impl PostmortemSlotRule for PostmortemSlotNamespaceLengthRule {
    fn check(&self, item: &PostmortemSlot) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSlotDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemSlotRule for PostmortemSlotDiscountRangeRule {
    fn check(&self, item: &PostmortemSlot) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a postmortem slot.
#[derive(Debug, Clone)]
pub struct PostmortemSlotValidator {
    pub rules: Vec<Rc<dyn PostmortemSlotRule>>,
}

impl PostmortemSlotValidator {
    pub fn validate(&self, item: &PostmortemSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_slot_validator() -> PostmortemSlotValidator {
    PostmortemSlotValidator {
        rules: vec![
            Rc::new(PostmortemSlotRequiredSourceRule {
                message: "postmortem slot needs an entry".to_string(),
            }) as Rc<dyn PostmortemSlotRule>,
            Rc::new(PostmortemSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PostmortemSlotNamespaceLengthRule { max_length: 255 }),
            Rc::new(PostmortemSlotDiscountRangeRule { min: 0.7, max: 4149.1 }),
        ],
    }
}
