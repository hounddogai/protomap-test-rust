use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped identity slot must pass. Check returns its problems, or none.
pub trait CappedIdentitySlotRule: std::fmt::Debug {
    fn check(&self, item: &CappedIdentitySlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIdentitySlotCodePrefixRule {
    pub prefix: String,
}

impl CappedIdentitySlotRule for CappedIdentitySlotCodePrefixRule {
    fn check(&self, item: &CappedIdentitySlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIdentitySlotRequiredNotesRule {
    pub message: String,
}

impl CappedIdentitySlotRule for CappedIdentitySlotRequiredNotesRule {
    fn check(&self, item: &CappedIdentitySlot) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIdentitySlotTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedIdentitySlotRule for CappedIdentitySlotTtlSecondsRangeRule {
    fn check(&self, item: &CappedIdentitySlot) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIdentitySlotFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedIdentitySlotRule for CappedIdentitySlotFloorValueRangeRule {
    fn check(&self, item: &CappedIdentitySlot) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIdentitySlotCodeLengthRule {
    pub max_length: i64,
}

impl CappedIdentitySlotRule for CappedIdentitySlotCodeLengthRule {
    fn check(&self, item: &CappedIdentitySlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIdentitySlotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedIdentitySlotRule for CappedIdentitySlotAllowedCodeRule {
    fn check(&self, item: &CappedIdentitySlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped identity slot.
#[derive(Debug, Clone)]
pub struct CappedIdentitySlotValidator {
    pub rules: Vec<Rc<dyn CappedIdentitySlotRule>>,
}

impl CappedIdentitySlotValidator {
    pub fn validate(&self, item: &CappedIdentitySlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedIdentitySlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedIdentitySlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_identity_slot_validator() -> CappedIdentitySlotValidator {
    CappedIdentitySlotValidator {
        rules: vec![
            Rc::new(CappedIdentitySlotCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn CappedIdentitySlotRule>,
            Rc::new(CappedIdentitySlotRequiredNotesRule {
                message: "capped identity slot needs this field".to_string(),
            }),
            Rc::new(CappedIdentitySlotTtlSecondsRangeRule { min: 80, max: 3600 }),
            Rc::new(CappedIdentitySlotFloorValueRangeRule { min: 6.9, max: 3847.3 }),
            Rc::new(CappedIdentitySlotCodeLengthRule { max_length: 32 }),
            Rc::new(CappedIdentitySlotAllowedCodeRule {
                allowed: vec!["pending".to_string()],
            }),
        ],
    }
}
