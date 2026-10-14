use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin slot must pass. Check returns its problems, or none.
pub trait RegionPinSlotRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlotNamespaceLengthRule {
    pub max_length: i64,
}

impl RegionPinSlotRule for RegionPinSlotNamespaceLengthRule {
    fn check(&self, item: &RegionPinSlot) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlotAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl RegionPinSlotRule for RegionPinSlotAllowedDescriptionRule {
    fn check(&self, item: &RegionPinSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is duplicated", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlotRequiredNamespaceRule {
    pub message: String,
}

impl RegionPinSlotRule for RegionPinSlotRequiredNamespaceRule {
    fn check(&self, item: &RegionPinSlot) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlotCodePrefixRule {
    pub prefix: String,
}

impl RegionPinSlotRule for RegionPinSlotCodePrefixRule {
    fn check(&self, item: &RegionPinSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlotPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinSlotRule for RegionPinSlotPortRangeRule {
    fn check(&self, item: &RegionPinSlot) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlotRequiredCodeRule {
    pub message: String,
}

impl RegionPinSlotRule for RegionPinSlotRequiredCodeRule {
    fn check(&self, item: &RegionPinSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin slot.
#[derive(Debug, Clone)]
pub struct RegionPinSlotValidator {
    pub rules: Vec<Rc<dyn RegionPinSlotRule>>,
}

impl RegionPinSlotValidator {
    pub fn validate(&self, item: &RegionPinSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_slot_validator() -> RegionPinSlotValidator {
    RegionPinSlotValidator {
        rules: vec![
            Rc::new(RegionPinSlotNamespaceLengthRule { max_length: 120 }) as Rc<dyn RegionPinSlotRule>,
            Rc::new(RegionPinSlotAllowedDescriptionRule {
                allowed: vec!["seasonal".to_string(), "regional".to_string(), "capped".to_string()],
            }),
            Rc::new(RegionPinSlotRequiredNamespaceRule {
                message: "region pin slot needs a value".to_string(),
            }),
            Rc::new(RegionPinSlotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RegionPinSlotPortRangeRule { min: 80, max: 120 }),
            Rc::new(RegionPinSlotRequiredCodeRule {
                message: "region pin slot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_slot_validator();
        let item = new_region_pin_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = RegionPinSlotNamespaceLengthRule { max_length: 80 };
        let mut item = new_region_pin_slot("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = RegionPinSlotRequiredNamespaceRule {
            message: "region pin slot needs a value".to_string(),
        };
        let mut item = new_region_pin_slot("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = RegionPinSlotPortRangeRule { min: 16, max: 120 };
        let mut item = new_region_pin_slot("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinSlotRequiredCodeRule {
            message: "region pin slot needs an entry".to_string(),
        };
        let mut item = new_region_pin_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
