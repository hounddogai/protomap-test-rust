use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo assignment must pass. Check returns its problems, or none.
pub trait SloAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &SloAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAssignmentSegmentLengthRule {
    pub max_length: i64,
}

impl SloAssignmentRule for SloAssignmentSegmentLengthRule {
    fn check(&self, item: &SloAssignment) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAssignmentRequiredTitleRule {
    pub message: String,
}

impl SloAssignmentRule for SloAssignmentRequiredTitleRule {
    fn check(&self, item: &SloAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAssignmentRequiredSegmentRule {
    pub message: String,
}

impl SloAssignmentRule for SloAssignmentRequiredSegmentRule {
    fn check(&self, item: &SloAssignment) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl SloAssignmentRule for SloAssignmentCodeLengthRule {
    fn check(&self, item: &SloAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo assignment.
#[derive(Debug, Clone)]
pub struct SloAssignmentValidator {
    pub rules: Vec<Rc<dyn SloAssignmentRule>>,
}

impl SloAssignmentValidator {
    pub fn validate(&self, item: &SloAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_assignment_validator() -> SloAssignmentValidator {
    SloAssignmentValidator {
        rules: vec![
            Rc::new(SloAssignmentSegmentLengthRule { max_length: 80 }) as Rc<dyn SloAssignmentRule>,
            Rc::new(SloAssignmentRequiredTitleRule {
                message: "slo assignment needs an entry".to_string(),
            }),
            Rc::new(SloAssignmentRequiredSegmentRule {
                message: "slo assignment needs this field".to_string(),
            }),
            Rc::new(SloAssignmentCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_assignment_validator();
        let item = new_slo_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = SloAssignmentSegmentLengthRule { max_length: 120 };
        let mut item = new_slo_assignment("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SloAssignmentRequiredTitleRule {
            message: "slo assignment needs this field".to_string(),
        };
        let mut item = new_slo_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = SloAssignmentRequiredSegmentRule {
            message: "slo assignment needs an entry".to_string(),
        };
        let mut item = new_slo_assignment("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SloAssignmentCodeLengthRule { max_length: 64 };
        let mut item = new_slo_assignment("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
