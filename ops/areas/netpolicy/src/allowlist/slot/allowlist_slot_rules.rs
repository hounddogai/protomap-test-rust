use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist slot must pass. Check returns its problems, or none.
pub trait AllowlistSlotRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSlotRequiredAudienceRule {
    pub message: String,
}

impl AllowlistSlotRule for AllowlistSlotRequiredAudienceRule {
    fn check(&self, item: &AllowlistSlot) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSlotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AllowlistSlotRule for AllowlistSlotAllowedCodeRule {
    fn check(&self, item: &AllowlistSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSlotNameLengthRule {
    pub max_length: i64,
}

impl AllowlistSlotRule for AllowlistSlotNameLengthRule {
    fn check(&self, item: &AllowlistSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist slot.
#[derive(Debug, Clone)]
pub struct AllowlistSlotValidator {
    pub rules: Vec<Rc<dyn AllowlistSlotRule>>,
}

impl AllowlistSlotValidator {
    pub fn validate(&self, item: &AllowlistSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_slot_validator() -> AllowlistSlotValidator {
    AllowlistSlotValidator {
        rules: vec![
            Rc::new(AllowlistSlotRequiredAudienceRule {
                message: "allowlist slot needs an entry".to_string(),
            }) as Rc<dyn AllowlistSlotRule>,
            Rc::new(AllowlistSlotAllowedCodeRule {
                allowed: vec!["default".to_string(), "scheduled".to_string()],
            }),
            Rc::new(AllowlistSlotNameLengthRule { max_length: 86400 }),
        ],
    }
}
