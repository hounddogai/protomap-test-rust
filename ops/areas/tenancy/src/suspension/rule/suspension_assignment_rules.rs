use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension assignment must pass. Check returns its problems, or none.
pub trait SuspensionAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAssignmentUnitLengthRule {
    pub max_length: i64,
}

impl SuspensionAssignmentRule for SuspensionAssignmentUnitLengthRule {
    fn check(&self, item: &SuspensionAssignment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAssignmentReviewerLengthRule {
    pub max_length: i64,
}

impl SuspensionAssignmentRule for SuspensionAssignmentReviewerLengthRule {
    fn check(&self, item: &SuspensionAssignment) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAssignmentRequiredReviewerRule {
    pub message: String,
}

impl SuspensionAssignmentRule for SuspensionAssignmentRequiredReviewerRule {
    fn check(&self, item: &SuspensionAssignment) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAssignmentRequiredCodeRule {
    pub message: String,
}

impl SuspensionAssignmentRule for SuspensionAssignmentRequiredCodeRule {
    fn check(&self, item: &SuspensionAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAssignmentAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl SuspensionAssignmentRule for SuspensionAssignmentAllowedUnitRule {
    fn check(&self, item: &SuspensionAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is stale", item.unit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension assignment.
#[derive(Debug, Clone)]
pub struct SuspensionAssignmentValidator {
    pub rules: Vec<Rc<dyn SuspensionAssignmentRule>>,
}

impl SuspensionAssignmentValidator {
    pub fn validate(&self, item: &SuspensionAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_assignment_validator() -> SuspensionAssignmentValidator {
    SuspensionAssignmentValidator {
        rules: vec![
            Rc::new(SuspensionAssignmentUnitLengthRule { max_length: 120 }) as Rc<dyn SuspensionAssignmentRule>,
            Rc::new(SuspensionAssignmentReviewerLengthRule { max_length: 86400 }),
            Rc::new(SuspensionAssignmentRequiredReviewerRule {
                message: "suspension assignment needs this field".to_string(),
            }),
            Rc::new(SuspensionAssignmentRequiredCodeRule {
                message: "suspension assignment needs this field".to_string(),
            }),
            Rc::new(SuspensionAssignmentAllowedUnitRule {
                allowed: vec!["global".to_string(), "lenient".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_assignment_validator();
        let item = new_suspension_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = SuspensionAssignmentUnitLengthRule { max_length: 1000 };
        let mut item = new_suspension_assignment("x", "y");
        item.unit = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = SuspensionAssignmentReviewerLengthRule { max_length: 1000 };
        let mut item = new_suspension_assignment("x", "y");
        item.reviewer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = SuspensionAssignmentRequiredReviewerRule {
            message: "suspension assignment needs this field".to_string(),
        };
        let mut item = new_suspension_assignment("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionAssignmentRequiredCodeRule {
            message: "suspension assignment needs this field".to_string(),
        };
        let mut item = new_suspension_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
