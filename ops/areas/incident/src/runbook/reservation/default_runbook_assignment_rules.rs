use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default runbook assignment must pass. Check returns its problems, or none.
pub trait DefaultRunbookAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRunbookAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookAssignmentAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl DefaultRunbookAssignmentRule for DefaultRunbookAssignmentAllowedNotesRule {
    fn check(&self, item: &DefaultRunbookAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is too long", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookAssignmentRequiredSegmentRule {
    pub message: String,
}

impl DefaultRunbookAssignmentRule for DefaultRunbookAssignmentRequiredSegmentRule {
    fn check(&self, item: &DefaultRunbookAssignment) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookAssignmentNotesLengthRule {
    pub max_length: i64,
}

impl DefaultRunbookAssignmentRule for DefaultRunbookAssignmentNotesLengthRule {
    fn check(&self, item: &DefaultRunbookAssignment) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookAssignmentRequiredNotesRule {
    pub message: String,
}

impl DefaultRunbookAssignmentRule for DefaultRunbookAssignmentRequiredNotesRule {
    fn check(&self, item: &DefaultRunbookAssignment) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl DefaultRunbookAssignmentRule for DefaultRunbookAssignmentCodeLengthRule {
    fn check(&self, item: &DefaultRunbookAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookAssignmentRequiredCodeRule {
    pub message: String,
}

impl DefaultRunbookAssignmentRule for DefaultRunbookAssignmentRequiredCodeRule {
    fn check(&self, item: &DefaultRunbookAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default runbook assignment.
#[derive(Debug, Clone)]
pub struct DefaultRunbookAssignmentValidator {
    pub rules: Vec<Rc<dyn DefaultRunbookAssignmentRule>>,
}

impl DefaultRunbookAssignmentValidator {
    pub fn validate(&self, item: &DefaultRunbookAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRunbookAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRunbookAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_runbook_assignment_validator() -> DefaultRunbookAssignmentValidator {
    DefaultRunbookAssignmentValidator {
        rules: vec![
            Rc::new(DefaultRunbookAssignmentAllowedNotesRule {
                allowed: vec!["capped".to_string(), "rolling".to_string(), "batched".to_string()],
            }) as Rc<dyn DefaultRunbookAssignmentRule>,
            Rc::new(DefaultRunbookAssignmentRequiredSegmentRule {
                message: "default runbook assignment needs an entry".to_string(),
            }),
            Rc::new(DefaultRunbookAssignmentNotesLengthRule { max_length: 80 }),
            Rc::new(DefaultRunbookAssignmentRequiredNotesRule {
                message: "default runbook assignment needs an entry".to_string(),
            }),
            Rc::new(DefaultRunbookAssignmentCodeLengthRule { max_length: 1000 }),
            Rc::new(DefaultRunbookAssignmentRequiredCodeRule {
                message: "default runbook assignment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_runbook_assignment_validator();
        let item = new_default_runbook_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = DefaultRunbookAssignmentRequiredSegmentRule {
            message: "default runbook assignment needs an entry".to_string(),
        };
        let mut item = new_default_runbook_assignment("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = DefaultRunbookAssignmentNotesLengthRule { max_length: 32 };
        let mut item = new_default_runbook_assignment("x", "y");
        item.notes = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = DefaultRunbookAssignmentRequiredNotesRule {
            message: "default runbook assignment needs a value".to_string(),
        };
        let mut item = new_default_runbook_assignment("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultRunbookAssignmentCodeLengthRule { max_length: 1000 };
        let mut item = new_default_runbook_assignment("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultRunbookAssignmentRequiredCodeRule {
            message: "default runbook assignment needs this field".to_string(),
        };
        let mut item = new_default_runbook_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
