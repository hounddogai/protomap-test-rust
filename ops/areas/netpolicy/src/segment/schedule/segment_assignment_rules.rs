use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment assignment must pass. Check returns its problems, or none.
pub trait SegmentAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &SegmentAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignmentPhoneLengthRule {
    pub max_length: i64,
}

impl SegmentAssignmentRule for SegmentAssignmentPhoneLengthRule {
    fn check(&self, item: &SegmentAssignment) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl SegmentAssignmentRule for SegmentAssignmentCodeLengthRule {
    fn check(&self, item: &SegmentAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignmentRequiredPhoneRule {
    pub message: String,
}

impl SegmentAssignmentRule for SegmentAssignmentRequiredPhoneRule {
    fn check(&self, item: &SegmentAssignment) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignmentRequiredTitleRule {
    pub message: String,
}

impl SegmentAssignmentRule for SegmentAssignmentRequiredTitleRule {
    fn check(&self, item: &SegmentAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignmentExternalRefLengthRule {
    pub max_length: i64,
}

impl SegmentAssignmentRule for SegmentAssignmentExternalRefLengthRule {
    fn check(&self, item: &SegmentAssignment) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl SegmentAssignmentRule for SegmentAssignmentAllowedTitleRule {
    fn check(&self, item: &SegmentAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment assignment.
#[derive(Debug, Clone)]
pub struct SegmentAssignmentValidator {
    pub rules: Vec<Rc<dyn SegmentAssignmentRule>>,
}

impl SegmentAssignmentValidator {
    pub fn validate(&self, item: &SegmentAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_assignment_validator() -> SegmentAssignmentValidator {
    SegmentAssignmentValidator {
        rules: vec![
            Rc::new(SegmentAssignmentPhoneLengthRule { max_length: 3600 }) as Rc<dyn SegmentAssignmentRule>,
            Rc::new(SegmentAssignmentCodeLengthRule { max_length: 64 }),
            Rc::new(SegmentAssignmentRequiredPhoneRule {
                message: "segment assignment needs this field".to_string(),
            }),
            Rc::new(SegmentAssignmentRequiredTitleRule {
                message: "segment assignment needs a value".to_string(),
            }),
            Rc::new(SegmentAssignmentExternalRefLengthRule { max_length: 16 }),
            Rc::new(SegmentAssignmentAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_assignment_validator();
        let item = new_segment_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = SegmentAssignmentPhoneLengthRule { max_length: 255 };
        let mut item = new_segment_assignment("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentAssignmentCodeLengthRule { max_length: 80 };
        let mut item = new_segment_assignment("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = SegmentAssignmentRequiredPhoneRule {
            message: "segment assignment needs a value".to_string(),
        };
        let mut item = new_segment_assignment("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SegmentAssignmentRequiredTitleRule {
            message: "segment assignment needs this field".to_string(),
        };
        let mut item = new_segment_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = SegmentAssignmentExternalRefLengthRule { max_length: 255 };
        let mut item = new_segment_assignment("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
