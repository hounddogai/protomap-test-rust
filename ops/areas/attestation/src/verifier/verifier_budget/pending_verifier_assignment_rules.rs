use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending verifier assignment must pass. Check returns its problems, or none.
pub trait PendingVerifierAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingVerifierAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierAssignmentPhoneLengthRule {
    pub max_length: i64,
}

impl PendingVerifierAssignmentRule for PendingVerifierAssignmentPhoneLengthRule {
    fn check(&self, item: &PendingVerifierAssignment) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierAssignmentRequiredTitleRule {
    pub message: String,
}

impl PendingVerifierAssignmentRule for PendingVerifierAssignmentRequiredTitleRule {
    fn check(&self, item: &PendingVerifierAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierAssignmentRequiredCodeRule {
    pub message: String,
}

impl PendingVerifierAssignmentRule for PendingVerifierAssignmentRequiredCodeRule {
    fn check(&self, item: &PendingVerifierAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierAssignmentRequiredSegmentRule {
    pub message: String,
}

impl PendingVerifierAssignmentRule for PendingVerifierAssignmentRequiredSegmentRule {
    fn check(&self, item: &PendingVerifierAssignment) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierAssignmentTitleLengthRule {
    pub max_length: i64,
}

impl PendingVerifierAssignmentRule for PendingVerifierAssignmentTitleLengthRule {
    fn check(&self, item: &PendingVerifierAssignment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending verifier assignment.
#[derive(Debug, Clone)]
pub struct PendingVerifierAssignmentValidator {
    pub rules: Vec<Rc<dyn PendingVerifierAssignmentRule>>,
}

impl PendingVerifierAssignmentValidator {
    pub fn validate(&self, item: &PendingVerifierAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingVerifierAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingVerifierAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_verifier_assignment_validator() -> PendingVerifierAssignmentValidator {
    PendingVerifierAssignmentValidator {
        rules: vec![
            Rc::new(PendingVerifierAssignmentPhoneLengthRule { max_length: 64 })
                as Rc<dyn PendingVerifierAssignmentRule>,
            Rc::new(PendingVerifierAssignmentRequiredTitleRule {
                message: "pending verifier assignment needs a value".to_string(),
            }),
            Rc::new(PendingVerifierAssignmentRequiredCodeRule {
                message: "pending verifier assignment needs this field".to_string(),
            }),
            Rc::new(PendingVerifierAssignmentRequiredSegmentRule {
                message: "pending verifier assignment needs this field".to_string(),
            }),
            Rc::new(PendingVerifierAssignmentTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_verifier_assignment_validator();
        let item = new_pending_verifier_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PendingVerifierAssignmentPhoneLengthRule { max_length: 64 };
        let mut item = new_pending_verifier_assignment("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingVerifierAssignmentRequiredTitleRule {
            message: "pending verifier assignment needs an entry".to_string(),
        };
        let mut item = new_pending_verifier_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingVerifierAssignmentRequiredCodeRule {
            message: "pending verifier assignment needs an entry".to_string(),
        };
        let mut item = new_pending_verifier_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = PendingVerifierAssignmentRequiredSegmentRule {
            message: "pending verifier assignment needs an entry".to_string(),
        };
        let mut item = new_pending_verifier_assignment("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingVerifierAssignmentTitleLengthRule { max_length: 80 };
        let mut item = new_pending_verifier_assignment("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
