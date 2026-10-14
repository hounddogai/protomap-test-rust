use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending freeze assignment must pass. Check returns its problems, or none.
pub trait PendingFreezeAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingFreezeAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PendingFreezeAssignmentRule for PendingFreezeAssignmentAllowedTitleRule {
    fn check(&self, item: &PendingFreezeAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeAssignmentRequiredCodeRule {
    pub message: String,
}

impl PendingFreezeAssignmentRule for PendingFreezeAssignmentRequiredCodeRule {
    fn check(&self, item: &PendingFreezeAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl PendingFreezeAssignmentRule for PendingFreezeAssignmentCodeLengthRule {
    fn check(&self, item: &PendingFreezeAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending freeze assignment.
#[derive(Debug, Clone)]
pub struct PendingFreezeAssignmentValidator {
    pub rules: Vec<Rc<dyn PendingFreezeAssignmentRule>>,
}

impl PendingFreezeAssignmentValidator {
    pub fn validate(&self, item: &PendingFreezeAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingFreezeAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFreezeAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_freeze_assignment_validator() -> PendingFreezeAssignmentValidator {
    PendingFreezeAssignmentValidator {
        rules: vec![
            Rc::new(PendingFreezeAssignmentAllowedTitleRule {
                allowed: vec!["lenient".to_string(), "strict".to_string(), "primary".to_string()],
            }) as Rc<dyn PendingFreezeAssignmentRule>,
            Rc::new(PendingFreezeAssignmentRequiredCodeRule {
                message: "pending freeze assignment needs this field".to_string(),
            }),
            Rc::new(PendingFreezeAssignmentCodeLengthRule { max_length: 16 }),
        ],
    }
}
