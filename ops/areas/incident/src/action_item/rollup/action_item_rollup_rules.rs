use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item rollup must pass. Check returns its problems, or none.
pub trait ActionItemRollupRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRollupKeyIdLengthRule {
    pub max_length: i64,
}

impl ActionItemRollupRule for ActionItemRollupKeyIdLengthRule {
    fn check(&self, item: &ActionItemRollup) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRollupCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemRollupRule for ActionItemRollupCodeLengthRule {
    fn check(&self, item: &ActionItemRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRollupRequiredLabelRule {
    pub message: String,
}

impl ActionItemRollupRule for ActionItemRollupRequiredLabelRule {
    fn check(&self, item: &ActionItemRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRollupAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ActionItemRollupRule for ActionItemRollupAllowedLabelRule {
    fn check(&self, item: &ActionItemRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRollupLabelLengthRule {
    pub max_length: i64,
}

impl ActionItemRollupRule for ActionItemRollupLabelLengthRule {
    fn check(&self, item: &ActionItemRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item rollup.
#[derive(Debug, Clone)]
pub struct ActionItemRollupValidator {
    pub rules: Vec<Rc<dyn ActionItemRollupRule>>,
}

impl ActionItemRollupValidator {
    pub fn validate(&self, item: &ActionItemRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_rollup_validator() -> ActionItemRollupValidator {
    ActionItemRollupValidator {
        rules: vec![
            Rc::new(ActionItemRollupKeyIdLengthRule { max_length: 64 }) as Rc<dyn ActionItemRollupRule>,
            Rc::new(ActionItemRollupCodeLengthRule { max_length: 3600 }),
            Rc::new(ActionItemRollupRequiredLabelRule {
                message: "action item rollup needs this field".to_string(),
            }),
            Rc::new(ActionItemRollupAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(ActionItemRollupLabelLengthRule { max_length: 80 }),
        ],
    }
}
