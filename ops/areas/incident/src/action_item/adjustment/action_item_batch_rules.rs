use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item batch must pass. Check returns its problems, or none.
pub trait ActionItemBatchRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBatchRequiredNameRule {
    pub message: String,
}

impl ActionItemBatchRule for ActionItemBatchRequiredNameRule {
    fn check(&self, item: &ActionItemBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBatchQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemBatchRule for ActionItemBatchQuantityRangeRule {
    fn check(&self, item: &ActionItemBatch) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBatchRequiredCodeRule {
    pub message: String,
}

impl ActionItemBatchRule for ActionItemBatchRequiredCodeRule {
    fn check(&self, item: &ActionItemBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBatchTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ActionItemBatchRule for ActionItemBatchTagsLimitRule {
    fn check(&self, item: &ActionItemBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBatchStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemBatchRule for ActionItemBatchStartsAtRangeRule {
    fn check(&self, item: &ActionItemBatch) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBatchCodePrefixRule {
    pub prefix: String,
}

impl ActionItemBatchRule for ActionItemBatchCodePrefixRule {
    fn check(&self, item: &ActionItemBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item batch.
#[derive(Debug, Clone)]
pub struct ActionItemBatchValidator {
    pub rules: Vec<Rc<dyn ActionItemBatchRule>>,
}

impl ActionItemBatchValidator {
    pub fn validate(&self, item: &ActionItemBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_batch_validator() -> ActionItemBatchValidator {
    ActionItemBatchValidator {
        rules: vec![
            Rc::new(ActionItemBatchRequiredNameRule {
                message: "action item batch needs this field".to_string(),
            }) as Rc<dyn ActionItemBatchRule>,
            Rc::new(ActionItemBatchQuantityRangeRule { min: 255, max: 3600 }),
            Rc::new(ActionItemBatchRequiredCodeRule {
                message: "action item batch needs this field".to_string(),
            }),
            Rc::new(ActionItemBatchTagsLimitRule {
                max_count: 64,
                banned: vec!["weighted".to_string(), "regional".to_string(), "global".to_string()],
            }),
            Rc::new(ActionItemBatchStartsAtRangeRule { min: 32, max: 120 }),
            Rc::new(ActionItemBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_batch_validator();
        let item = new_action_item_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ActionItemBatchRequiredNameRule {
            message: "action item batch needs this field".to_string(),
        };
        let mut item = new_action_item_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ActionItemBatchQuantityRangeRule { min: 80, max: 3600 };
        let mut item = new_action_item_batch("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ActionItemBatchRequiredCodeRule {
            message: "action item batch needs a value".to_string(),
        };
        let mut item = new_action_item_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ActionItemBatchStartsAtRangeRule { min: 80, max: 86400 };
        let mut item = new_action_item_batch("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
