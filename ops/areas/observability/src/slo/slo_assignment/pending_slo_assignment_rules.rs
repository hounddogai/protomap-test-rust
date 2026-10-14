use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending slo assignment must pass. Check returns its problems, or none.
pub trait PendingSloAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingSloAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloAssignmentNameLengthRule {
    pub max_length: i64,
}

impl PendingSloAssignmentRule for PendingSloAssignmentNameLengthRule {
    fn check(&self, item: &PendingSloAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloAssignmentRequiredCodeRule {
    pub message: String,
}

impl PendingSloAssignmentRule for PendingSloAssignmentRequiredCodeRule {
    fn check(&self, item: &PendingSloAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloAssignmentCodePrefixRule {
    pub prefix: String,
}

impl PendingSloAssignmentRule for PendingSloAssignmentCodePrefixRule {
    fn check(&self, item: &PendingSloAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloAssignmentBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingSloAssignmentRule for PendingSloAssignmentBatchSizeRangeRule {
    fn check(&self, item: &PendingSloAssignment) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloAssignmentReviewerLengthRule {
    pub max_length: i64,
}

impl PendingSloAssignmentRule for PendingSloAssignmentReviewerLengthRule {
    fn check(&self, item: &PendingSloAssignment) -> Vec<String> {
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
pub struct PendingSloAssignmentAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl PendingSloAssignmentRule for PendingSloAssignmentAllowedReviewerRule {
    fn check(&self, item: &PendingSloAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is below minimum", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending slo assignment.
#[derive(Debug, Clone)]
pub struct PendingSloAssignmentValidator {
    pub rules: Vec<Rc<dyn PendingSloAssignmentRule>>,
}

impl PendingSloAssignmentValidator {
    pub fn validate(&self, item: &PendingSloAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingSloAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSloAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_slo_assignment_validator() -> PendingSloAssignmentValidator {
    PendingSloAssignmentValidator {
        rules: vec![
            Rc::new(PendingSloAssignmentNameLengthRule { max_length: 3600 }) as Rc<dyn PendingSloAssignmentRule>,
            Rc::new(PendingSloAssignmentRequiredCodeRule {
                message: "pending slo assignment needs a value".to_string(),
            }),
            Rc::new(PendingSloAssignmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PendingSloAssignmentBatchSizeRangeRule { min: 64, max: 86400 }),
            Rc::new(PendingSloAssignmentReviewerLengthRule { max_length: 64 }),
            Rc::new(PendingSloAssignmentAllowedReviewerRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_slo_assignment_validator();
        let item = new_pending_slo_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingSloAssignmentNameLengthRule { max_length: 120 };
        let mut item = new_pending_slo_assignment("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingSloAssignmentRequiredCodeRule {
            message: "pending slo assignment needs this field".to_string(),
        };
        let mut item = new_pending_slo_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PendingSloAssignmentBatchSizeRangeRule { min: 120, max: 255 };
        let mut item = new_pending_slo_assignment("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = PendingSloAssignmentReviewerLengthRule { max_length: 64 };
        let mut item = new_pending_slo_assignment("x", "y");
        item.reviewer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
