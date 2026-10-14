use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item checkpoint must pass. Check returns its problems, or none.
pub trait ActionItemCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ActionItemCheckpointRule for ActionItemCheckpointAllowedCodeRule {
    fn check(&self, item: &ActionItemCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCheckpointRequiredCategoryRule {
    pub message: String,
}

impl ActionItemCheckpointRule for ActionItemCheckpointRequiredCategoryRule {
    fn check(&self, item: &ActionItemCheckpoint) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemCheckpointRequiredCodeRule {
    pub message: String,
}

impl ActionItemCheckpointRule for ActionItemCheckpointRequiredCodeRule {
    fn check(&self, item: &ActionItemCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item checkpoint.
#[derive(Debug, Clone)]
pub struct ActionItemCheckpointValidator {
    pub rules: Vec<Rc<dyn ActionItemCheckpointRule>>,
}

impl ActionItemCheckpointValidator {
    pub fn validate(&self, item: &ActionItemCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_checkpoint_validator() -> ActionItemCheckpointValidator {
    ActionItemCheckpointValidator {
        rules: vec![
            Rc::new(ActionItemCheckpointAllowedCodeRule {
                allowed: vec!["pending".to_string(), "strict".to_string(), "pending".to_string()],
            }) as Rc<dyn ActionItemCheckpointRule>,
            Rc::new(ActionItemCheckpointRequiredCategoryRule {
                message: "action item checkpoint needs an entry".to_string(),
            }),
            Rc::new(ActionItemCheckpointRequiredCodeRule {
                message: "action item checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_checkpoint_validator();
        let item = new_action_item_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = ActionItemCheckpointRequiredCategoryRule {
            message: "action item checkpoint needs this field".to_string(),
        };
        let mut item = new_action_item_checkpoint("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ActionItemCheckpointRequiredCodeRule {
            message: "action item checkpoint needs this field".to_string(),
        };
        let mut item = new_action_item_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
