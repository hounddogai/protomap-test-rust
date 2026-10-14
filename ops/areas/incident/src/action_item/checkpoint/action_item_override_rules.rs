use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item override must pass. Check returns its problems, or none.
pub trait ActionItemOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemOverrideDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemOverrideRule for ActionItemOverrideDiscountRangeRule {
    fn check(&self, item: &ActionItemOverride) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemOverrideRequiredPhoneRule {
    pub message: String,
}

impl ActionItemOverrideRule for ActionItemOverrideRequiredPhoneRule {
    fn check(&self, item: &ActionItemOverride) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemOverrideCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemOverrideRule for ActionItemOverrideCodeLengthRule {
    fn check(&self, item: &ActionItemOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemOverrideRequiredNameRule {
    pub message: String,
}

impl ActionItemOverrideRule for ActionItemOverrideRequiredNameRule {
    fn check(&self, item: &ActionItemOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemOverrideArchivedGuardRule {
    pub limit: f64,
}

impl ActionItemOverrideRule for ActionItemOverrideArchivedGuardRule {
    fn check(&self, item: &ActionItemOverride) -> Vec<String> {
        if item.archived && (item.ceiling > self.limit) {
            return vec![format!(
                "ceiling {} exceeds {} while archived",
                item.ceiling, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item override.
#[derive(Debug, Clone)]
pub struct ActionItemOverrideValidator {
    pub rules: Vec<Rc<dyn ActionItemOverrideRule>>,
}

impl ActionItemOverrideValidator {
    pub fn validate(&self, item: &ActionItemOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_override_validator() -> ActionItemOverrideValidator {
    ActionItemOverrideValidator {
        rules: vec![
            Rc::new(ActionItemOverrideDiscountRangeRule { min: 9.9, max: 4536.4 }) as Rc<dyn ActionItemOverrideRule>,
            Rc::new(ActionItemOverrideRequiredPhoneRule {
                message: "action item override needs a value".to_string(),
            }),
            Rc::new(ActionItemOverrideCodeLengthRule { max_length: 1000 }),
            Rc::new(ActionItemOverrideRequiredNameRule {
                message: "action item override needs a value".to_string(),
            }),
            Rc::new(ActionItemOverrideArchivedGuardRule { limit: 4.4 }),
        ],
    }
}
