use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem assignment must pass. Check returns its problems, or none.
pub trait PostmortemAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAssignmentAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PostmortemAssignmentRule for PostmortemAssignmentAllowedLabelRule {
    fn check(&self, item: &PostmortemAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAssignmentRequiredNamespaceRule {
    pub message: String,
}

impl PostmortemAssignmentRule for PostmortemAssignmentRequiredNamespaceRule {
    fn check(&self, item: &PostmortemAssignment) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAssignmentAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemAssignmentRule for PostmortemAssignmentAttemptsRangeRule {
    fn check(&self, item: &PostmortemAssignment) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem assignment.
#[derive(Debug, Clone)]
pub struct PostmortemAssignmentValidator {
    pub rules: Vec<Rc<dyn PostmortemAssignmentRule>>,
}

impl PostmortemAssignmentValidator {
    pub fn validate(&self, item: &PostmortemAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_assignment_validator() -> PostmortemAssignmentValidator {
    PostmortemAssignmentValidator {
        rules: vec![
            Rc::new(PostmortemAssignmentAllowedLabelRule {
                allowed: vec!["primary".to_string(), "regional".to_string(), "scheduled".to_string()],
            }) as Rc<dyn PostmortemAssignmentRule>,
            Rc::new(PostmortemAssignmentRequiredNamespaceRule {
                message: "postmortem assignment needs this field".to_string(),
            }),
            Rc::new(PostmortemAssignmentAttemptsRangeRule { min: 64, max: 1000 }),
        ],
    }
}
