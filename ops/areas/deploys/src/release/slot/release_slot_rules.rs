use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release slot must pass. Check returns its problems, or none.
pub trait ReleaseSlotRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSlotRequiredCodeRule {
    pub message: String,
}

impl ReleaseSlotRule for ReleaseSlotRequiredCodeRule {
    fn check(&self, item: &ReleaseSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSlotCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseSlotRule for ReleaseSlotCodeLengthRule {
    fn check(&self, item: &ReleaseSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSlotLabelLengthRule {
    pub max_length: i64,
}

impl ReleaseSlotRule for ReleaseSlotLabelLengthRule {
    fn check(&self, item: &ReleaseSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSlotOwnerIdLengthRule {
    pub max_length: i64,
}

impl ReleaseSlotRule for ReleaseSlotOwnerIdLengthRule {
    fn check(&self, item: &ReleaseSlot) -> Vec<String> {
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

/// Runs every rule of a release slot.
#[derive(Debug, Clone)]
pub struct ReleaseSlotValidator {
    pub rules: Vec<Rc<dyn ReleaseSlotRule>>,
}

impl ReleaseSlotValidator {
    pub fn validate(&self, item: &ReleaseSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_slot_validator() -> ReleaseSlotValidator {
    ReleaseSlotValidator {
        rules: vec![
            Rc::new(ReleaseSlotRequiredCodeRule {
                message: "release slot needs this field".to_string(),
            }) as Rc<dyn ReleaseSlotRule>,
            Rc::new(ReleaseSlotCodeLengthRule { max_length: 120 }),
            Rc::new(ReleaseSlotLabelLengthRule { max_length: 1000 }),
            Rc::new(ReleaseSlotOwnerIdLengthRule { max_length: 32 }),
        ],
    }
}
