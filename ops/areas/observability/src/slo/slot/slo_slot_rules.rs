use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo slot must pass. Check returns its problems, or none.
pub trait SloSlotRule: std::fmt::Debug {
    fn check(&self, item: &SloSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSlotRequiredTitleRule {
    pub message: String,
}

impl SloSlotRule for SloSlotRequiredTitleRule {
    fn check(&self, item: &SloSlot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSlotRequiredSegmentRule {
    pub message: String,
}

impl SloSlotRule for SloSlotRequiredSegmentRule {
    fn check(&self, item: &SloSlot) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSlotRequiredCodeRule {
    pub message: String,
}

impl SloSlotRule for SloSlotRequiredCodeRule {
    fn check(&self, item: &SloSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSlotCodeLengthRule {
    pub max_length: i64,
}

impl SloSlotRule for SloSlotCodeLengthRule {
    fn check(&self, item: &SloSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloSlotAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl SloSlotRule for SloSlotAllowedSegmentRule {
    fn check(&self, item: &SloSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is not allowed", item.segment)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo slot.
#[derive(Debug, Clone)]
pub struct SloSlotValidator {
    pub rules: Vec<Rc<dyn SloSlotRule>>,
}

impl SloSlotValidator {
    pub fn validate(&self, item: &SloSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_slot_validator() -> SloSlotValidator {
    SloSlotValidator {
        rules: vec![
            Rc::new(SloSlotRequiredTitleRule {
                message: "slo slot needs a value".to_string(),
            }) as Rc<dyn SloSlotRule>,
            Rc::new(SloSlotRequiredSegmentRule {
                message: "slo slot needs an entry".to_string(),
            }),
            Rc::new(SloSlotRequiredCodeRule {
                message: "slo slot needs a value".to_string(),
            }),
            Rc::new(SloSlotCodeLengthRule { max_length: 64 }),
            Rc::new(SloSlotAllowedSegmentRule { allowed: Vec::new() }),
        ],
    }
}
