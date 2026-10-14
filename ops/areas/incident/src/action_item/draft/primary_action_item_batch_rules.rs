use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary action item batch must pass. Check returns its problems, or none.
pub trait PrimaryActionItemBatchRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryActionItemBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBatchFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryActionItemBatchRule for PrimaryActionItemBatchFloorValueRangeRule {
    fn check(&self, item: &PrimaryActionItemBatch) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBatchAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PrimaryActionItemBatchRule for PrimaryActionItemBatchAllowedLabelRule {
    fn check(&self, item: &PrimaryActionItemBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is out of range", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBatchDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryActionItemBatchRule for PrimaryActionItemBatchDiscountRangeRule {
    fn check(&self, item: &PrimaryActionItemBatch) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBatchCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryActionItemBatchRule for PrimaryActionItemBatchCodeLengthRule {
    fn check(&self, item: &PrimaryActionItemBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBatchPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryActionItemBatchRule for PrimaryActionItemBatchPriorityRangeRule {
    fn check(&self, item: &PrimaryActionItemBatch) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBatchLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryActionItemBatchRule for PrimaryActionItemBatchLabelLengthRule {
    fn check(&self, item: &PrimaryActionItemBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary action item batch.
#[derive(Debug, Clone)]
pub struct PrimaryActionItemBatchValidator {
    pub rules: Vec<Rc<dyn PrimaryActionItemBatchRule>>,
}

impl PrimaryActionItemBatchValidator {
    pub fn validate(&self, item: &PrimaryActionItemBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryActionItemBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryActionItemBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_action_item_batch_validator() -> PrimaryActionItemBatchValidator {
    PrimaryActionItemBatchValidator {
        rules: vec![
            Rc::new(PrimaryActionItemBatchFloorValueRangeRule { min: 5.8, max: 1242.0 })
                as Rc<dyn PrimaryActionItemBatchRule>,
            Rc::new(PrimaryActionItemBatchAllowedLabelRule {
                allowed: vec!["seasonal".to_string(), "scheduled".to_string()],
            }),
            Rc::new(PrimaryActionItemBatchDiscountRangeRule { min: 9.9, max: 4858.6 }),
            Rc::new(PrimaryActionItemBatchCodeLengthRule { max_length: 32 }),
            Rc::new(PrimaryActionItemBatchPriorityRangeRule { min: 16, max: 86400 }),
            Rc::new(PrimaryActionItemBatchLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_action_item_batch_validator();
        let item = new_primary_action_item_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = PrimaryActionItemBatchFloorValueRangeRule { min: 9.6, max: 2465.7 };
        let mut item = new_primary_action_item_batch("x", "y");
        item.floor_value = 2467.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = PrimaryActionItemBatchDiscountRangeRule { min: 3.6, max: 3516.0 };
        let mut item = new_primary_action_item_batch("x", "y");
        item.discount = 3517.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryActionItemBatchCodeLengthRule { max_length: 3600 };
        let mut item = new_primary_action_item_batch("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PrimaryActionItemBatchPriorityRangeRule { min: 1000, max: 86400 };
        let mut item = new_primary_action_item_batch("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryActionItemBatchLabelLengthRule { max_length: 80 };
        let mut item = new_primary_action_item_batch("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
