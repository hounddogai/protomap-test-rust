use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item milestone must pass. Check returns its problems, or none.
pub trait ActionItemMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemMilestoneRule for ActionItemMilestoneCodeLengthRule {
    fn check(&self, item: &ActionItemMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMilestoneCodePrefixRule {
    pub prefix: String,
}

impl ActionItemMilestoneRule for ActionItemMilestoneCodePrefixRule {
    fn check(&self, item: &ActionItemMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMilestoneUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemMilestoneRule for ActionItemMilestoneUpdatedAtRangeRule {
    fn check(&self, item: &ActionItemMilestone) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMilestoneDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemMilestoneRule for ActionItemMilestoneDurationMinutesRangeRule {
    fn check(&self, item: &ActionItemMilestone) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMilestoneRequiredLabelRule {
    pub message: String,
}

impl ActionItemMilestoneRule for ActionItemMilestoneRequiredLabelRule {
    fn check(&self, item: &ActionItemMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMilestoneRequiredCodeRule {
    pub message: String,
}

impl ActionItemMilestoneRule for ActionItemMilestoneRequiredCodeRule {
    fn check(&self, item: &ActionItemMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item milestone.
#[derive(Debug, Clone)]
pub struct ActionItemMilestoneValidator {
    pub rules: Vec<Rc<dyn ActionItemMilestoneRule>>,
}

impl ActionItemMilestoneValidator {
    pub fn validate(&self, item: &ActionItemMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_milestone_validator() -> ActionItemMilestoneValidator {
    ActionItemMilestoneValidator {
        rules: vec![
            Rc::new(ActionItemMilestoneCodeLengthRule { max_length: 120 }) as Rc<dyn ActionItemMilestoneRule>,
            Rc::new(ActionItemMilestoneCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ActionItemMilestoneUpdatedAtRangeRule { min: 16, max: 3600 }),
            Rc::new(ActionItemMilestoneDurationMinutesRangeRule { min: 255, max: 3600 }),
            Rc::new(ActionItemMilestoneRequiredLabelRule {
                message: "action item milestone needs a value".to_string(),
            }),
            Rc::new(ActionItemMilestoneRequiredCodeRule {
                message: "action item milestone needs an entry".to_string(),
            }),
        ],
    }
}
