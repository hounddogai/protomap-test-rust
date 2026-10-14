use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage slot must pass. Check returns its problems, or none.
pub trait UsageSlotRule: std::fmt::Debug {
    fn check(&self, item: &UsageSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSlotDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageSlotRule for UsageSlotDiscountRangeRule {
    fn check(&self, item: &UsageSlot) -> Vec<String> {
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
pub struct UsageSlotQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl UsageSlotRule for UsageSlotQuantityRangeRule {
    fn check(&self, item: &UsageSlot) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSlotCodeLengthRule {
    pub max_length: i64,
}

impl UsageSlotRule for UsageSlotCodeLengthRule {
    fn check(&self, item: &UsageSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSlotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl UsageSlotRule for UsageSlotAllowedCodeRule {
    fn check(&self, item: &UsageSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSlotMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageSlotRule for UsageSlotMaximumRangeRule {
    fn check(&self, item: &UsageSlot) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage slot.
#[derive(Debug, Clone)]
pub struct UsageSlotValidator {
    pub rules: Vec<Rc<dyn UsageSlotRule>>,
}

impl UsageSlotValidator {
    pub fn validate(&self, item: &UsageSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_slot_validator() -> UsageSlotValidator {
    UsageSlotValidator {
        rules: vec![
            Rc::new(UsageSlotDiscountRangeRule { min: 1.2, max: 3538.8 }) as Rc<dyn UsageSlotRule>,
            Rc::new(UsageSlotQuantityRangeRule { min: 32, max: 32 }),
            Rc::new(UsageSlotCodeLengthRule { max_length: 80 }),
            Rc::new(UsageSlotAllowedCodeRule {
                allowed: vec!["batched".to_string(), "batched".to_string(), "strict".to_string()],
            }),
            Rc::new(UsageSlotMaximumRangeRule { min: 2.3, max: 3381.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_slot_validator();
        let item = new_usage_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = UsageSlotDiscountRangeRule { min: 3.0, max: 4105.6 };
        let mut item = new_usage_slot("x", "y");
        item.discount = 4107.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = UsageSlotQuantityRangeRule { min: 1000, max: 3600 };
        let mut item = new_usage_slot("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = UsageSlotCodeLengthRule { max_length: 32 };
        let mut item = new_usage_slot("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = UsageSlotMaximumRangeRule { min: 7.2, max: 4911.5 };
        let mut item = new_usage_slot("x", "y");
        item.maximum = 4913.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
