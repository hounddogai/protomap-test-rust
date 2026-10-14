use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default reservation slot must pass. Check returns its problems, or none.
pub trait DefaultReservationSlotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultReservationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSlotNameLengthRule {
    pub max_length: i64,
}

impl DefaultReservationSlotRule for DefaultReservationSlotNameLengthRule {
    fn check(&self, item: &DefaultReservationSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSlotAllowedNameRule {
    pub allowed: Vec<String>,
}

impl DefaultReservationSlotRule for DefaultReservationSlotAllowedNameRule {
    fn check(&self, item: &DefaultReservationSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSlotRequiredCategoryRule {
    pub message: String,
}

impl DefaultReservationSlotRule for DefaultReservationSlotRequiredCategoryRule {
    fn check(&self, item: &DefaultReservationSlot) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSlotNamespaceLengthRule {
    pub max_length: i64,
}

impl DefaultReservationSlotRule for DefaultReservationSlotNamespaceLengthRule {
    fn check(&self, item: &DefaultReservationSlot) -> Vec<String> {
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
pub struct DefaultReservationSlotCodeLengthRule {
    pub max_length: i64,
}

impl DefaultReservationSlotRule for DefaultReservationSlotCodeLengthRule {
    fn check(&self, item: &DefaultReservationSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSlotRequiredNamespaceRule {
    pub message: String,
}

impl DefaultReservationSlotRule for DefaultReservationSlotRequiredNamespaceRule {
    fn check(&self, item: &DefaultReservationSlot) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default reservation slot.
#[derive(Debug, Clone)]
pub struct DefaultReservationSlotValidator {
    pub rules: Vec<Rc<dyn DefaultReservationSlotRule>>,
}

impl DefaultReservationSlotValidator {
    pub fn validate(&self, item: &DefaultReservationSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultReservationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultReservationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_reservation_slot_validator() -> DefaultReservationSlotValidator {
    DefaultReservationSlotValidator {
        rules: vec![
            Rc::new(DefaultReservationSlotNameLengthRule { max_length: 3600 }) as Rc<dyn DefaultReservationSlotRule>,
            Rc::new(DefaultReservationSlotAllowedNameRule { allowed: Vec::new() }),
            Rc::new(DefaultReservationSlotRequiredCategoryRule {
                message: "default reservation slot needs an entry".to_string(),
            }),
            Rc::new(DefaultReservationSlotNamespaceLengthRule { max_length: 86400 }),
            Rc::new(DefaultReservationSlotCodeLengthRule { max_length: 120 }),
            Rc::new(DefaultReservationSlotRequiredNamespaceRule {
                message: "default reservation slot needs a value".to_string(),
            }),
        ],
    }
}
