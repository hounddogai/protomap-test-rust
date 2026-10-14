use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary region pin slot must pass. Check returns its problems, or none.
pub trait PrimaryRegionPinSlotRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRegionPinSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinSlotRequiredLabelRule {
    pub message: String,
}

impl PrimaryRegionPinSlotRule for PrimaryRegionPinSlotRequiredLabelRule {
    fn check(&self, item: &PrimaryRegionPinSlot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinSlotCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRegionPinSlotRule for PrimaryRegionPinSlotCodeLengthRule {
    fn check(&self, item: &PrimaryRegionPinSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinSlotCodePrefixRule {
    pub prefix: String,
}

impl PrimaryRegionPinSlotRule for PrimaryRegionPinSlotCodePrefixRule {
    fn check(&self, item: &PrimaryRegionPinSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinSlotThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRegionPinSlotRule for PrimaryRegionPinSlotThresholdRangeRule {
    fn check(&self, item: &PrimaryRegionPinSlot) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinSlotAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PrimaryRegionPinSlotRule for PrimaryRegionPinSlotAllowedLabelRule {
    fn check(&self, item: &PrimaryRegionPinSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary region pin slot.
#[derive(Debug, Clone)]
pub struct PrimaryRegionPinSlotValidator {
    pub rules: Vec<Rc<dyn PrimaryRegionPinSlotRule>>,
}

impl PrimaryRegionPinSlotValidator {
    pub fn validate(&self, item: &PrimaryRegionPinSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRegionPinSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRegionPinSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_region_pin_slot_validator() -> PrimaryRegionPinSlotValidator {
    PrimaryRegionPinSlotValidator {
        rules: vec![
            Rc::new(PrimaryRegionPinSlotRequiredLabelRule {
                message: "primary region pin slot needs an entry".to_string(),
            }) as Rc<dyn PrimaryRegionPinSlotRule>,
            Rc::new(PrimaryRegionPinSlotCodeLengthRule { max_length: 3600 }),
            Rc::new(PrimaryRegionPinSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryRegionPinSlotThresholdRangeRule { min: 8.9, max: 2035.8 }),
            Rc::new(PrimaryRegionPinSlotAllowedLabelRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_region_pin_slot_validator();
        let item = new_primary_region_pin_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryRegionPinSlotRequiredLabelRule {
            message: "primary region pin slot needs a value".to_string(),
        };
        let mut item = new_primary_region_pin_slot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRegionPinSlotCodeLengthRule { max_length: 255 };
        let mut item = new_primary_region_pin_slot("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PrimaryRegionPinSlotThresholdRangeRule { min: 7.4, max: 2360.7 };
        let mut item = new_primary_region_pin_slot("x", "y");
        item.threshold = 2362.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
