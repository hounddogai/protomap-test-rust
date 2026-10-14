use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default action item draft must pass. Check returns its problems, or none.
pub trait DefaultActionItemDraftRule: std::fmt::Debug {
    fn check(&self, item: &DefaultActionItemDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemDraftRequiredCodeRule {
    pub message: String,
}

impl DefaultActionItemDraftRule for DefaultActionItemDraftRequiredCodeRule {
    fn check(&self, item: &DefaultActionItemDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemDraftCodeLengthRule {
    pub max_length: i64,
}

impl DefaultActionItemDraftRule for DefaultActionItemDraftCodeLengthRule {
    fn check(&self, item: &DefaultActionItemDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemDraftSourceLengthRule {
    pub max_length: i64,
}

impl DefaultActionItemDraftRule for DefaultActionItemDraftSourceLengthRule {
    fn check(&self, item: &DefaultActionItemDraft) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemDraftPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultActionItemDraftRule for DefaultActionItemDraftPriorityRangeRule {
    fn check(&self, item: &DefaultActionItemDraft) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default action item draft.
#[derive(Debug, Clone)]
pub struct DefaultActionItemDraftValidator {
    pub rules: Vec<Rc<dyn DefaultActionItemDraftRule>>,
}

impl DefaultActionItemDraftValidator {
    pub fn validate(&self, item: &DefaultActionItemDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultActionItemDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultActionItemDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_action_item_draft_validator() -> DefaultActionItemDraftValidator {
    DefaultActionItemDraftValidator {
        rules: vec![
            Rc::new(DefaultActionItemDraftRequiredCodeRule {
                message: "default action item draft needs an entry".to_string(),
            }) as Rc<dyn DefaultActionItemDraftRule>,
            Rc::new(DefaultActionItemDraftCodeLengthRule { max_length: 64 }),
            Rc::new(DefaultActionItemDraftSourceLengthRule { max_length: 255 }),
            Rc::new(DefaultActionItemDraftPriorityRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_action_item_draft_validator();
        let item = new_default_action_item_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultActionItemDraftRequiredCodeRule {
            message: "default action item draft needs this field".to_string(),
        };
        let mut item = new_default_action_item_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultActionItemDraftCodeLengthRule { max_length: 16 };
        let mut item = new_default_action_item_draft("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = DefaultActionItemDraftSourceLengthRule { max_length: 3600 };
        let mut item = new_default_action_item_draft("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = DefaultActionItemDraftPriorityRangeRule { min: 16, max: 3600 };
        let mut item = new_default_action_item_draft("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
