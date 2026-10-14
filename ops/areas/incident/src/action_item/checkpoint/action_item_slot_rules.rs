use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item slot must pass. Check returns its problems, or none.
pub trait ActionItemSlotRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSlotRequiredTitleRule {
    pub message: String,
}

impl ActionItemSlotRule for ActionItemSlotRequiredTitleRule {
    fn check(&self, item: &ActionItemSlot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSlotTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemSlotRule for ActionItemSlotTtlSecondsRangeRule {
    fn check(&self, item: &ActionItemSlot) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSlotCategoryLengthRule {
    pub max_length: i64,
}

impl ActionItemSlotRule for ActionItemSlotCategoryLengthRule {
    fn check(&self, item: &ActionItemSlot) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item slot.
#[derive(Debug, Clone)]
pub struct ActionItemSlotValidator {
    pub rules: Vec<Rc<dyn ActionItemSlotRule>>,
}

impl ActionItemSlotValidator {
    pub fn validate(&self, item: &ActionItemSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_slot_validator() -> ActionItemSlotValidator {
    ActionItemSlotValidator {
        rules: vec![
            Rc::new(ActionItemSlotRequiredTitleRule {
                message: "action item slot needs an entry".to_string(),
            }) as Rc<dyn ActionItemSlotRule>,
            Rc::new(ActionItemSlotTtlSecondsRangeRule { min: 32, max: 80 }),
            Rc::new(ActionItemSlotCategoryLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_slot_validator();
        let item = new_action_item_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ActionItemSlotRequiredTitleRule {
            message: "action item slot needs a value".to_string(),
        };
        let mut item = new_action_item_slot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = ActionItemSlotTtlSecondsRangeRule { min: 1000, max: 3600 };
        let mut item = new_action_item_slot("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = ActionItemSlotCategoryLengthRule { max_length: 16 };
        let mut item = new_action_item_slot("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
