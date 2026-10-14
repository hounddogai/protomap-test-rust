use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item assignment must pass. Check returns its problems, or none.
pub trait ActionItemAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemAssignmentRule for ActionItemAssignmentCodeLengthRule {
    fn check(&self, item: &ActionItemAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAssignmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemAssignmentRule for ActionItemAssignmentGraceDaysRangeRule {
    fn check(&self, item: &ActionItemAssignment) -> Vec<String> {
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
pub struct ActionItemAssignmentDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemAssignmentRule for ActionItemAssignmentDurationMinutesRangeRule {
    fn check(&self, item: &ActionItemAssignment) -> Vec<String> {
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
pub struct ActionItemAssignmentCodePrefixRule {
    pub prefix: String,
}

impl ActionItemAssignmentRule for ActionItemAssignmentCodePrefixRule {
    fn check(&self, item: &ActionItemAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAssignmentRequiredChecksumRule {
    pub message: String,
}

impl ActionItemAssignmentRule for ActionItemAssignmentRequiredChecksumRule {
    fn check(&self, item: &ActionItemAssignment) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item assignment.
#[derive(Debug, Clone)]
pub struct ActionItemAssignmentValidator {
    pub rules: Vec<Rc<dyn ActionItemAssignmentRule>>,
}

impl ActionItemAssignmentValidator {
    pub fn validate(&self, item: &ActionItemAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_assignment_validator() -> ActionItemAssignmentValidator {
    ActionItemAssignmentValidator {
        rules: vec![
            Rc::new(ActionItemAssignmentCodeLengthRule { max_length: 16 }) as Rc<dyn ActionItemAssignmentRule>,
            Rc::new(ActionItemAssignmentGraceDaysRangeRule { min: 255, max: 3600 }),
            Rc::new(ActionItemAssignmentDurationMinutesRangeRule { min: 3600, max: 86400 }),
            Rc::new(ActionItemAssignmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ActionItemAssignmentRequiredChecksumRule {
                message: "action item assignment needs an entry".to_string(),
            }),
        ],
    }
}
