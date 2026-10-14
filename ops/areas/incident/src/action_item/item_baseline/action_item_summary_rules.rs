use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item summary must pass. Check returns its problems, or none.
pub trait ActionItemSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSummaryCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemSummaryRule for ActionItemSummaryCapacityRangeRule {
    fn check(&self, item: &ActionItemSummary) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSummaryCipherLengthRule {
    pub max_length: i64,
}

impl ActionItemSummaryRule for ActionItemSummaryCipherLengthRule {
    fn check(&self, item: &ActionItemSummary) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSummaryTitleLengthRule {
    pub max_length: i64,
}

impl ActionItemSummaryRule for ActionItemSummaryTitleLengthRule {
    fn check(&self, item: &ActionItemSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSummaryCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemSummaryRule for ActionItemSummaryCodeLengthRule {
    fn check(&self, item: &ActionItemSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSummaryCodePrefixRule {
    pub prefix: String,
}

impl ActionItemSummaryRule for ActionItemSummaryCodePrefixRule {
    fn check(&self, item: &ActionItemSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSummaryRequiredTitleRule {
    pub message: String,
}

impl ActionItemSummaryRule for ActionItemSummaryRequiredTitleRule {
    fn check(&self, item: &ActionItemSummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item summary.
#[derive(Debug, Clone)]
pub struct ActionItemSummaryValidator {
    pub rules: Vec<Rc<dyn ActionItemSummaryRule>>,
}

impl ActionItemSummaryValidator {
    pub fn validate(&self, item: &ActionItemSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_summary_validator() -> ActionItemSummaryValidator {
    ActionItemSummaryValidator {
        rules: vec![
            Rc::new(ActionItemSummaryCapacityRangeRule { min: 120, max: 1000 }) as Rc<dyn ActionItemSummaryRule>,
            Rc::new(ActionItemSummaryCipherLengthRule { max_length: 32 }),
            Rc::new(ActionItemSummaryTitleLengthRule { max_length: 86400 }),
            Rc::new(ActionItemSummaryCodeLengthRule { max_length: 80 }),
            Rc::new(ActionItemSummaryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ActionItemSummaryRequiredTitleRule {
                message: "action item summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_summary_validator();
        let item = new_action_item_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = ActionItemSummaryCapacityRangeRule { min: 64, max: 1000 };
        let mut item = new_action_item_summary("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = ActionItemSummaryCipherLengthRule { max_length: 32 };
        let mut item = new_action_item_summary("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ActionItemSummaryTitleLengthRule { max_length: 64 };
        let mut item = new_action_item_summary("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ActionItemSummaryCodeLengthRule { max_length: 32 };
        let mut item = new_action_item_summary("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ActionItemSummaryRequiredTitleRule {
            message: "action item summary needs a value".to_string(),
        };
        let mut item = new_action_item_summary("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
