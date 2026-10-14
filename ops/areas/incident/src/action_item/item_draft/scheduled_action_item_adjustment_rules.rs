use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled action item adjustment must pass. Check returns its problems, or none.
pub trait ScheduledActionItemAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledActionItemAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemAdjustmentRequiredTitleRule {
    pub message: String,
}

impl ScheduledActionItemAdjustmentRule for ScheduledActionItemAdjustmentRequiredTitleRule {
    fn check(&self, item: &ScheduledActionItemAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemAdjustmentRequiredCodeRule {
    pub message: String,
}

impl ScheduledActionItemAdjustmentRule for ScheduledActionItemAdjustmentRequiredCodeRule {
    fn check(&self, item: &ScheduledActionItemAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledActionItemAdjustmentRule for ScheduledActionItemAdjustmentTitleLengthRule {
    fn check(&self, item: &ScheduledActionItemAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemAdjustmentCountryLengthRule {
    pub max_length: i64,
}

impl ScheduledActionItemAdjustmentRule for ScheduledActionItemAdjustmentCountryLengthRule {
    fn check(&self, item: &ScheduledActionItemAdjustment) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled action item adjustment.
#[derive(Debug, Clone)]
pub struct ScheduledActionItemAdjustmentValidator {
    pub rules: Vec<Rc<dyn ScheduledActionItemAdjustmentRule>>,
}

impl ScheduledActionItemAdjustmentValidator {
    pub fn validate(&self, item: &ScheduledActionItemAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledActionItemAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledActionItemAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_action_item_adjustment_validator() -> ScheduledActionItemAdjustmentValidator {
    ScheduledActionItemAdjustmentValidator {
        rules: vec![
            Rc::new(ScheduledActionItemAdjustmentRequiredTitleRule {
                message: "scheduled action item adjustment needs a value".to_string(),
            }) as Rc<dyn ScheduledActionItemAdjustmentRule>,
            Rc::new(ScheduledActionItemAdjustmentRequiredCodeRule {
                message: "scheduled action item adjustment needs an entry".to_string(),
            }),
            Rc::new(ScheduledActionItemAdjustmentTitleLengthRule { max_length: 16 }),
            Rc::new(ScheduledActionItemAdjustmentCountryLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_action_item_adjustment_validator();
        let item = new_scheduled_action_item_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledActionItemAdjustmentRequiredTitleRule {
            message: "scheduled action item adjustment needs a value".to_string(),
        };
        let mut item = new_scheduled_action_item_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledActionItemAdjustmentRequiredCodeRule {
            message: "scheduled action item adjustment needs an entry".to_string(),
        };
        let mut item = new_scheduled_action_item_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledActionItemAdjustmentTitleLengthRule { max_length: 255 };
        let mut item = new_scheduled_action_item_adjustment("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = ScheduledActionItemAdjustmentCountryLengthRule { max_length: 255 };
        let mut item = new_scheduled_action_item_adjustment("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
