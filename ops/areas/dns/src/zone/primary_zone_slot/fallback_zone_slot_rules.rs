use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback zone slot must pass. Check returns its problems, or none.
pub trait FallbackZoneSlotRule: std::fmt::Debug {
    fn check(&self, item: &FallbackZoneSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSlotAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl FallbackZoneSlotRule for FallbackZoneSlotAllowedAudienceRule {
    fn check(&self, item: &FallbackZoneSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is stale", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSlotMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackZoneSlotRule for FallbackZoneSlotMarginRangeRule {
    fn check(&self, item: &FallbackZoneSlot) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSlotCodeLengthRule {
    pub max_length: i64,
}

impl FallbackZoneSlotRule for FallbackZoneSlotCodeLengthRule {
    fn check(&self, item: &FallbackZoneSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSlotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackZoneSlotRule for FallbackZoneSlotUpdatedAtRangeRule {
    fn check(&self, item: &FallbackZoneSlot) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSlotRequiredSourceRule {
    pub message: String,
}

impl FallbackZoneSlotRule for FallbackZoneSlotRequiredSourceRule {
    fn check(&self, item: &FallbackZoneSlot) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback zone slot.
#[derive(Debug, Clone)]
pub struct FallbackZoneSlotValidator {
    pub rules: Vec<Rc<dyn FallbackZoneSlotRule>>,
}

impl FallbackZoneSlotValidator {
    pub fn validate(&self, item: &FallbackZoneSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackZoneSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackZoneSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_zone_slot_validator() -> FallbackZoneSlotValidator {
    FallbackZoneSlotValidator {
        rules: vec![
            Rc::new(FallbackZoneSlotAllowedAudienceRule { allowed: Vec::new() }) as Rc<dyn FallbackZoneSlotRule>,
            Rc::new(FallbackZoneSlotMarginRangeRule { min: 8.7, max: 2158.6 }),
            Rc::new(FallbackZoneSlotCodeLengthRule { max_length: 3600 }),
            Rc::new(FallbackZoneSlotUpdatedAtRangeRule { min: 16, max: 86400 }),
            Rc::new(FallbackZoneSlotRequiredSourceRule {
                message: "fallback zone slot needs a value".to_string(),
            }),
        ],
    }
}
