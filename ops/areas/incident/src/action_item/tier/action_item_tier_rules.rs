use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item tier must pass. Check returns its problems, or none.
pub trait ActionItemTierRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTierLabelLengthRule {
    pub max_length: i64,
}

impl ActionItemTierRule for ActionItemTierLabelLengthRule {
    fn check(&self, item: &ActionItemTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTierRequiredCodeRule {
    pub message: String,
}

impl ActionItemTierRule for ActionItemTierRequiredCodeRule {
    fn check(&self, item: &ActionItemTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTierGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemTierRule for ActionItemTierGraceDaysRangeRule {
    fn check(&self, item: &ActionItemTier) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTierRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemTierRule for ActionItemTierRotationDaysRangeRule {
    fn check(&self, item: &ActionItemTier) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTierVisibleGuardRule {
    pub limit: i64,
}

impl ActionItemTierRule for ActionItemTierVisibleGuardRule {
    fn check(&self, item: &ActionItemTier) -> Vec<String> {
        if item.visible && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while visible", item.limit, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item tier.
#[derive(Debug, Clone)]
pub struct ActionItemTierValidator {
    pub rules: Vec<Rc<dyn ActionItemTierRule>>,
}

impl ActionItemTierValidator {
    pub fn validate(&self, item: &ActionItemTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_tier_validator() -> ActionItemTierValidator {
    ActionItemTierValidator {
        rules: vec![
            Rc::new(ActionItemTierLabelLengthRule { max_length: 86400 }) as Rc<dyn ActionItemTierRule>,
            Rc::new(ActionItemTierRequiredCodeRule {
                message: "action item tier needs a value".to_string(),
            }),
            Rc::new(ActionItemTierGraceDaysRangeRule { min: 80, max: 120 }),
            Rc::new(ActionItemTierRotationDaysRangeRule { min: 120, max: 3600 }),
            Rc::new(ActionItemTierVisibleGuardRule { limit: 255 }),
        ],
    }
}
