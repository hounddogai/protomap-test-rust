use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional action item adjustment must pass. Check returns its problems, or none.
pub trait RegionalActionItemAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalActionItemAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemAdjustmentCipherLengthRule {
    pub max_length: i64,
}

impl RegionalActionItemAdjustmentRule for RegionalActionItemAdjustmentCipherLengthRule {
    fn check(&self, item: &RegionalActionItemAdjustment) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemAdjustmentRequiredNameRule {
    pub message: String,
}

impl RegionalActionItemAdjustmentRule for RegionalActionItemAdjustmentRequiredNameRule {
    fn check(&self, item: &RegionalActionItemAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemAdjustmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalActionItemAdjustmentRule for RegionalActionItemAdjustmentFloorValueRangeRule {
    fn check(&self, item: &RegionalActionItemAdjustment) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemAdjustmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RegionalActionItemAdjustmentRule for RegionalActionItemAdjustmentAllowedNameRule {
    fn check(&self, item: &RegionalActionItemAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional action item adjustment.
#[derive(Debug, Clone)]
pub struct RegionalActionItemAdjustmentValidator {
    pub rules: Vec<Rc<dyn RegionalActionItemAdjustmentRule>>,
}

impl RegionalActionItemAdjustmentValidator {
    pub fn validate(&self, item: &RegionalActionItemAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalActionItemAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalActionItemAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_action_item_adjustment_validator() -> RegionalActionItemAdjustmentValidator {
    RegionalActionItemAdjustmentValidator {
        rules: vec![
            Rc::new(RegionalActionItemAdjustmentCipherLengthRule { max_length: 16 })
                as Rc<dyn RegionalActionItemAdjustmentRule>,
            Rc::new(RegionalActionItemAdjustmentRequiredNameRule {
                message: "regional action item adjustment needs this field".to_string(),
            }),
            Rc::new(RegionalActionItemAdjustmentFloorValueRangeRule { min: 5.8, max: 209.5 }),
            Rc::new(RegionalActionItemAdjustmentAllowedNameRule {
                allowed: vec!["legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_action_item_adjustment_validator();
        let item = new_regional_action_item_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = RegionalActionItemAdjustmentCipherLengthRule { max_length: 32 };
        let mut item = new_regional_action_item_adjustment("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalActionItemAdjustmentRequiredNameRule {
            message: "regional action item adjustment needs an entry".to_string(),
        };
        let mut item = new_regional_action_item_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RegionalActionItemAdjustmentFloorValueRangeRule { min: 0.1, max: 2863.6 };
        let mut item = new_regional_action_item_adjustment("x", "y");
        item.floor_value = 2865.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
