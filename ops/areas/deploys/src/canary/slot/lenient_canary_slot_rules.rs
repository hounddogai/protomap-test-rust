use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient canary slot must pass. Check returns its problems, or none.
pub trait LenientCanarySlotRule: std::fmt::Debug {
    fn check(&self, item: &LenientCanarySlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanarySlotRequiredTimezoneRule {
    pub message: String,
}

impl LenientCanarySlotRule for LenientCanarySlotRequiredTimezoneRule {
    fn check(&self, item: &LenientCanarySlot) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanarySlotAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl LenientCanarySlotRule for LenientCanarySlotAllowedTimezoneRule {
    fn check(&self, item: &LenientCanarySlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is inconsistent", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanarySlotMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientCanarySlotRule for LenientCanarySlotMinimumRangeRule {
    fn check(&self, item: &LenientCanarySlot) -> Vec<String> {
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

/// Runs every rule of a lenient canary slot.
#[derive(Debug, Clone)]
pub struct LenientCanarySlotValidator {
    pub rules: Vec<Rc<dyn LenientCanarySlotRule>>,
}

impl LenientCanarySlotValidator {
    pub fn validate(&self, item: &LenientCanarySlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientCanarySlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientCanarySlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_canary_slot_validator() -> LenientCanarySlotValidator {
    LenientCanarySlotValidator {
        rules: vec![
            Rc::new(LenientCanarySlotRequiredTimezoneRule {
                message: "lenient canary slot needs this field".to_string(),
            }) as Rc<dyn LenientCanarySlotRule>,
            Rc::new(LenientCanarySlotAllowedTimezoneRule { allowed: Vec::new() }),
            Rc::new(LenientCanarySlotMinimumRangeRule { min: 7.8, max: 2271.8 }),
        ],
    }
}
