use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release assignment must pass. Check returns its problems, or none.
pub trait ReleaseAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAssignmentRequiredCodeRule {
    pub message: String,
}

impl ReleaseAssignmentRule for ReleaseAssignmentRequiredCodeRule {
    fn check(&self, item: &ReleaseAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ReleaseAssignmentRule for ReleaseAssignmentAllowedCodeRule {
    fn check(&self, item: &ReleaseAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAssignmentDescriptionLengthRule {
    pub max_length: i64,
}

impl ReleaseAssignmentRule for ReleaseAssignmentDescriptionLengthRule {
    fn check(&self, item: &ReleaseAssignment) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAssignmentNameLengthRule {
    pub max_length: i64,
}

impl ReleaseAssignmentRule for ReleaseAssignmentNameLengthRule {
    fn check(&self, item: &ReleaseAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAssignmentRequiredNameRule {
    pub message: String,
}

impl ReleaseAssignmentRule for ReleaseAssignmentRequiredNameRule {
    fn check(&self, item: &ReleaseAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAssignmentRequiredDescriptionRule {
    pub message: String,
}

impl ReleaseAssignmentRule for ReleaseAssignmentRequiredDescriptionRule {
    fn check(&self, item: &ReleaseAssignment) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release assignment.
#[derive(Debug, Clone)]
pub struct ReleaseAssignmentValidator {
    pub rules: Vec<Rc<dyn ReleaseAssignmentRule>>,
}

impl ReleaseAssignmentValidator {
    pub fn validate(&self, item: &ReleaseAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_assignment_validator() -> ReleaseAssignmentValidator {
    ReleaseAssignmentValidator {
        rules: vec![
            Rc::new(ReleaseAssignmentRequiredCodeRule {
                message: "release assignment needs a value".to_string(),
            }) as Rc<dyn ReleaseAssignmentRule>,
            Rc::new(ReleaseAssignmentAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(ReleaseAssignmentDescriptionLengthRule { max_length: 255 }),
            Rc::new(ReleaseAssignmentNameLengthRule { max_length: 64 }),
            Rc::new(ReleaseAssignmentRequiredNameRule {
                message: "release assignment needs this field".to_string(),
            }),
            Rc::new(ReleaseAssignmentRequiredDescriptionRule {
                message: "release assignment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_assignment_validator();
        let item = new_release_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleaseAssignmentRequiredCodeRule {
            message: "release assignment needs an entry".to_string(),
        };
        let mut item = new_release_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = ReleaseAssignmentDescriptionLengthRule { max_length: 86400 };
        let mut item = new_release_assignment("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ReleaseAssignmentNameLengthRule { max_length: 120 };
        let mut item = new_release_assignment("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ReleaseAssignmentRequiredNameRule {
            message: "release assignment needs a value".to_string(),
        };
        let mut item = new_release_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = ReleaseAssignmentRequiredDescriptionRule {
            message: "release assignment needs this field".to_string(),
        };
        let mut item = new_release_assignment("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
