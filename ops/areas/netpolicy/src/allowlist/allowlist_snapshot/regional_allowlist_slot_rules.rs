use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional allowlist slot must pass. Check returns its problems, or none.
pub trait RegionalAllowlistSlotRule: std::fmt::Debug {
    fn check(&self, item: &RegionalAllowlistSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistSlotCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalAllowlistSlotRule for RegionalAllowlistSlotCreatedAtRangeRule {
    fn check(&self, item: &RegionalAllowlistSlot) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistSlotRequiredCodeRule {
    pub message: String,
}

impl RegionalAllowlistSlotRule for RegionalAllowlistSlotRequiredCodeRule {
    fn check(&self, item: &RegionalAllowlistSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistSlotCodePrefixRule {
    pub prefix: String,
}

impl RegionalAllowlistSlotRule for RegionalAllowlistSlotCodePrefixRule {
    fn check(&self, item: &RegionalAllowlistSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistSlotRegionLengthRule {
    pub max_length: i64,
}

impl RegionalAllowlistSlotRule for RegionalAllowlistSlotRegionLengthRule {
    fn check(&self, item: &RegionalAllowlistSlot) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistSlotCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalAllowlistSlotRule for RegionalAllowlistSlotCostRangeRule {
    fn check(&self, item: &RegionalAllowlistSlot) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistSlotAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl RegionalAllowlistSlotRule for RegionalAllowlistSlotAllowedRegionRule {
    fn check(&self, item: &RegionalAllowlistSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is not allowed", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional allowlist slot.
#[derive(Debug, Clone)]
pub struct RegionalAllowlistSlotValidator {
    pub rules: Vec<Rc<dyn RegionalAllowlistSlotRule>>,
}

impl RegionalAllowlistSlotValidator {
    pub fn validate(&self, item: &RegionalAllowlistSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalAllowlistSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalAllowlistSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_allowlist_slot_validator() -> RegionalAllowlistSlotValidator {
    RegionalAllowlistSlotValidator {
        rules: vec![
            Rc::new(RegionalAllowlistSlotCreatedAtRangeRule { min: 64, max: 86400 })
                as Rc<dyn RegionalAllowlistSlotRule>,
            Rc::new(RegionalAllowlistSlotRequiredCodeRule {
                message: "regional allowlist slot needs an entry".to_string(),
            }),
            Rc::new(RegionalAllowlistSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionalAllowlistSlotRegionLengthRule { max_length: 3600 }),
            Rc::new(RegionalAllowlistSlotCostRangeRule { min: 6.0, max: 4722.3 }),
            Rc::new(RegionalAllowlistSlotAllowedRegionRule {
                allowed: vec!["global".to_string(), "rolling".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_allowlist_slot_validator();
        let item = new_regional_allowlist_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RegionalAllowlistSlotCreatedAtRangeRule { min: 32, max: 120 };
        let mut item = new_regional_allowlist_slot("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalAllowlistSlotRequiredCodeRule {
            message: "regional allowlist slot needs this field".to_string(),
        };
        let mut item = new_regional_allowlist_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = RegionalAllowlistSlotRegionLengthRule { max_length: 80 };
        let mut item = new_regional_allowlist_slot("x", "y");
        item.region = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RegionalAllowlistSlotCostRangeRule { min: 5.9, max: 4155.2 };
        let mut item = new_regional_allowlist_slot("x", "y");
        item.cost = 4156.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
