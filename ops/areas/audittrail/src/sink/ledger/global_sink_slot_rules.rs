use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global sink slot must pass. Check returns its problems, or none.
pub trait GlobalSinkSlotRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSinkSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkSlotCodePrefixRule {
    pub prefix: String,
}

impl GlobalSinkSlotRule for GlobalSinkSlotCodePrefixRule {
    fn check(&self, item: &GlobalSinkSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkSlotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalSinkSlotRule for GlobalSinkSlotAllowedCodeRule {
    fn check(&self, item: &GlobalSinkSlot) -> Vec<String> {
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
pub struct GlobalSinkSlotCodeLengthRule {
    pub max_length: i64,
}

impl GlobalSinkSlotRule for GlobalSinkSlotCodeLengthRule {
    fn check(&self, item: &GlobalSinkSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkSlotNameLengthRule {
    pub max_length: i64,
}

impl GlobalSinkSlotRule for GlobalSinkSlotNameLengthRule {
    fn check(&self, item: &GlobalSinkSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkSlotFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalSinkSlotRule for GlobalSinkSlotFloorValueRangeRule {
    fn check(&self, item: &GlobalSinkSlot) -> Vec<String> {
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

/// Runs every rule of a global sink slot.
#[derive(Debug, Clone)]
pub struct GlobalSinkSlotValidator {
    pub rules: Vec<Rc<dyn GlobalSinkSlotRule>>,
}

impl GlobalSinkSlotValidator {
    pub fn validate(&self, item: &GlobalSinkSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalSinkSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSinkSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_sink_slot_validator() -> GlobalSinkSlotValidator {
    GlobalSinkSlotValidator {
        rules: vec![
            Rc::new(GlobalSinkSlotCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn GlobalSinkSlotRule>,
            Rc::new(GlobalSinkSlotAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(GlobalSinkSlotCodeLengthRule { max_length: 32 }),
            Rc::new(GlobalSinkSlotNameLengthRule { max_length: 86400 }),
            Rc::new(GlobalSinkSlotFloorValueRangeRule { min: 8.2, max: 4579.7 }),
        ],
    }
}
