use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched zone slot must pass. Check returns its problems, or none.
pub trait BatchedZoneSlotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedZoneSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneSlotNameLengthRule {
    pub max_length: i64,
}

impl BatchedZoneSlotRule for BatchedZoneSlotNameLengthRule {
    fn check(&self, item: &BatchedZoneSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneSlotRequiredNameRule {
    pub message: String,
}

impl BatchedZoneSlotRule for BatchedZoneSlotRequiredNameRule {
    fn check(&self, item: &BatchedZoneSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneSlotTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedZoneSlotRule for BatchedZoneSlotTtlSecondsRangeRule {
    fn check(&self, item: &BatchedZoneSlot) -> Vec<String> {
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
pub struct BatchedZoneSlotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedZoneSlotRule for BatchedZoneSlotAllowedCodeRule {
    fn check(&self, item: &BatchedZoneSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched zone slot.
#[derive(Debug, Clone)]
pub struct BatchedZoneSlotValidator {
    pub rules: Vec<Rc<dyn BatchedZoneSlotRule>>,
}

impl BatchedZoneSlotValidator {
    pub fn validate(&self, item: &BatchedZoneSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedZoneSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedZoneSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_zone_slot_validator() -> BatchedZoneSlotValidator {
    BatchedZoneSlotValidator {
        rules: vec![
            Rc::new(BatchedZoneSlotNameLengthRule { max_length: 64 }) as Rc<dyn BatchedZoneSlotRule>,
            Rc::new(BatchedZoneSlotRequiredNameRule {
                message: "batched zone slot needs an entry".to_string(),
            }),
            Rc::new(BatchedZoneSlotTtlSecondsRangeRule { min: 32, max: 255 }),
            Rc::new(BatchedZoneSlotAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "strict".to_string()],
            }),
        ],
    }
}
