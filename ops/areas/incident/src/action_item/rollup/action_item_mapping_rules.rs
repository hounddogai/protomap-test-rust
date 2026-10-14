use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item mapping must pass. Check returns its problems, or none.
pub trait ActionItemMappingRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMappingNamespaceLengthRule {
    pub max_length: i64,
}

impl ActionItemMappingRule for ActionItemMappingNamespaceLengthRule {
    fn check(&self, item: &ActionItemMapping) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMappingCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemMappingRule for ActionItemMappingCodeLengthRule {
    fn check(&self, item: &ActionItemMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMappingThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemMappingRule for ActionItemMappingThresholdRangeRule {
    fn check(&self, item: &ActionItemMapping) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMappingLabelLengthRule {
    pub max_length: i64,
}

impl ActionItemMappingRule for ActionItemMappingLabelLengthRule {
    fn check(&self, item: &ActionItemMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item mapping.
#[derive(Debug, Clone)]
pub struct ActionItemMappingValidator {
    pub rules: Vec<Rc<dyn ActionItemMappingRule>>,
}

impl ActionItemMappingValidator {
    pub fn validate(&self, item: &ActionItemMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_mapping_validator() -> ActionItemMappingValidator {
    ActionItemMappingValidator {
        rules: vec![
            Rc::new(ActionItemMappingNamespaceLengthRule { max_length: 1000 }) as Rc<dyn ActionItemMappingRule>,
            Rc::new(ActionItemMappingCodeLengthRule { max_length: 255 }),
            Rc::new(ActionItemMappingThresholdRangeRule { min: 3.2, max: 4079.9 }),
            Rc::new(ActionItemMappingLabelLengthRule { max_length: 32 }),
        ],
    }
}
