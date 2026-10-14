use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering slot must pass. Check returns its problems, or none.
pub trait PeeringSlotRule: std::fmt::Debug {
    fn check(&self, item: &PeeringSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSlotLabelLengthRule {
    pub max_length: i64,
}

impl PeeringSlotRule for PeeringSlotLabelLengthRule {
    fn check(&self, item: &PeeringSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSlotQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringSlotRule for PeeringSlotQuantityRangeRule {
    fn check(&self, item: &PeeringSlot) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSlotWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringSlotRule for PeeringSlotWindowDaysRangeRule {
    fn check(&self, item: &PeeringSlot) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSlotCodePrefixRule {
    pub prefix: String,
}

impl PeeringSlotRule for PeeringSlotCodePrefixRule {
    fn check(&self, item: &PeeringSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering slot.
#[derive(Debug, Clone)]
pub struct PeeringSlotValidator {
    pub rules: Vec<Rc<dyn PeeringSlotRule>>,
}

impl PeeringSlotValidator {
    pub fn validate(&self, item: &PeeringSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_slot_validator() -> PeeringSlotValidator {
    PeeringSlotValidator {
        rules: vec![
            Rc::new(PeeringSlotLabelLengthRule { max_length: 16 }) as Rc<dyn PeeringSlotRule>,
            Rc::new(PeeringSlotQuantityRangeRule { min: 80, max: 3600 }),
            Rc::new(PeeringSlotWindowDaysRangeRule { min: 255, max: 3600 }),
            Rc::new(PeeringSlotCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
