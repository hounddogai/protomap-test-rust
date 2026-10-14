use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient suspension assignment must pass. Check returns its problems, or none.
pub trait LenientSuspensionAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LenientSuspensionAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAssignmentUnitLengthRule {
    pub max_length: i64,
}

impl LenientSuspensionAssignmentRule for LenientSuspensionAssignmentUnitLengthRule {
    fn check(&self, item: &LenientSuspensionAssignment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAssignmentRequiredTitleRule {
    pub message: String,
}

impl LenientSuspensionAssignmentRule for LenientSuspensionAssignmentRequiredTitleRule {
    fn check(&self, item: &LenientSuspensionAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAssignmentRequiredCodeRule {
    pub message: String,
}

impl LenientSuspensionAssignmentRule for LenientSuspensionAssignmentRequiredCodeRule {
    fn check(&self, item: &LenientSuspensionAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAssignmentAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl LenientSuspensionAssignmentRule for LenientSuspensionAssignmentAllowedUnitRule {
    fn check(&self, item: &LenientSuspensionAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is blocked", item.unit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient suspension assignment.
#[derive(Debug, Clone)]
pub struct LenientSuspensionAssignmentValidator {
    pub rules: Vec<Rc<dyn LenientSuspensionAssignmentRule>>,
}

impl LenientSuspensionAssignmentValidator {
    pub fn validate(&self, item: &LenientSuspensionAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSuspensionAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSuspensionAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_suspension_assignment_validator() -> LenientSuspensionAssignmentValidator {
    LenientSuspensionAssignmentValidator {
        rules: vec![
            Rc::new(LenientSuspensionAssignmentUnitLengthRule { max_length: 86400 })
                as Rc<dyn LenientSuspensionAssignmentRule>,
            Rc::new(LenientSuspensionAssignmentRequiredTitleRule {
                message: "lenient suspension assignment needs a value".to_string(),
            }),
            Rc::new(LenientSuspensionAssignmentRequiredCodeRule {
                message: "lenient suspension assignment needs an entry".to_string(),
            }),
            Rc::new(LenientSuspensionAssignmentAllowedUnitRule {
                allowed: vec!["strict".to_string(), "rolling".to_string(), "primary".to_string()],
            }),
        ],
    }
}
