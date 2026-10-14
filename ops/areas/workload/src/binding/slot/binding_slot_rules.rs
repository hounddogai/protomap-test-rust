use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding slot must pass. Check returns its problems, or none.
pub trait BindingSlotRule: std::fmt::Debug {
    fn check(&self, item: &BindingSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlotAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl BindingSlotRule for BindingSlotAllowedCountryRule {
    fn check(&self, item: &BindingSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is out of range", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlotMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingSlotRule for BindingSlotMinimumRangeRule {
    fn check(&self, item: &BindingSlot) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlotSegmentLengthRule {
    pub max_length: i64,
}

impl BindingSlotRule for BindingSlotSegmentLengthRule {
    fn check(&self, item: &BindingSlot) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlotCountryLengthRule {
    pub max_length: i64,
}

impl BindingSlotRule for BindingSlotCountryLengthRule {
    fn check(&self, item: &BindingSlot) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlotRequiredSegmentRule {
    pub message: String,
}

impl BindingSlotRule for BindingSlotRequiredSegmentRule {
    fn check(&self, item: &BindingSlot) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlotRequiredTitleRule {
    pub message: String,
}

impl BindingSlotRule for BindingSlotRequiredTitleRule {
    fn check(&self, item: &BindingSlot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding slot.
#[derive(Debug, Clone)]
pub struct BindingSlotValidator {
    pub rules: Vec<Rc<dyn BindingSlotRule>>,
}

impl BindingSlotValidator {
    pub fn validate(&self, item: &BindingSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_slot_validator() -> BindingSlotValidator {
    BindingSlotValidator {
        rules: vec![
            Rc::new(BindingSlotAllowedCountryRule {
                allowed: vec!["scheduled".to_string()],
            }) as Rc<dyn BindingSlotRule>,
            Rc::new(BindingSlotMinimumRangeRule { min: 6.1, max: 3644.0 }),
            Rc::new(BindingSlotSegmentLengthRule { max_length: 1000 }),
            Rc::new(BindingSlotCountryLengthRule { max_length: 16 }),
            Rc::new(BindingSlotRequiredSegmentRule {
                message: "binding slot needs an entry".to_string(),
            }),
            Rc::new(BindingSlotRequiredTitleRule {
                message: "binding slot needs this field".to_string(),
            }),
        ],
    }
}
