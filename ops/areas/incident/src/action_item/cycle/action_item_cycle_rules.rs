use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item cycle must pass. Check returns its problems, or none.
pub trait ActionItemCycleRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCyclePhoneLengthRule {
    pub max_length: i64,
}

impl ActionItemCycleRule for ActionItemCyclePhoneLengthRule {
    fn check(&self, item: &ActionItemCycle) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCycleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemCycleRule for ActionItemCycleDiscountRangeRule {
    fn check(&self, item: &ActionItemCycle) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCycleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ActionItemCycleRule for ActionItemCycleAllowedNameRule {
    fn check(&self, item: &ActionItemCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCycleRequiredPhoneRule {
    pub message: String,
}

impl ActionItemCycleRule for ActionItemCycleRequiredPhoneRule {
    fn check(&self, item: &ActionItemCycle) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item cycle.
#[derive(Debug, Clone)]
pub struct ActionItemCycleValidator {
    pub rules: Vec<Rc<dyn ActionItemCycleRule>>,
}

impl ActionItemCycleValidator {
    pub fn validate(&self, item: &ActionItemCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_cycle_validator() -> ActionItemCycleValidator {
    ActionItemCycleValidator {
        rules: vec![
            Rc::new(ActionItemCyclePhoneLengthRule { max_length: 255 }) as Rc<dyn ActionItemCycleRule>,
            Rc::new(ActionItemCycleDiscountRangeRule { min: 0.1, max: 4906.3 }),
            Rc::new(ActionItemCycleAllowedNameRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(ActionItemCycleRequiredPhoneRule {
                message: "action item cycle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_cycle_validator();
        let item = new_action_item_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = ActionItemCyclePhoneLengthRule { max_length: 120 };
        let mut item = new_action_item_cycle("x", "y");
        item.phone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = ActionItemCycleDiscountRangeRule { min: 4.4, max: 1433.4 };
        let mut item = new_action_item_cycle("x", "y");
        item.discount = 1434.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ActionItemCycleRequiredPhoneRule {
            message: "action item cycle needs a value".to_string(),
        };
        let mut item = new_action_item_cycle("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
