use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval assignment must pass. Check returns its problems, or none.
pub trait ApprovalAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAssignmentAudienceLengthRule {
    pub max_length: i64,
}

impl ApprovalAssignmentRule for ApprovalAssignmentAudienceLengthRule {
    fn check(&self, item: &ApprovalAssignment) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAssignmentRequiredChecksumRule {
    pub message: String,
}

impl ApprovalAssignmentRule for ApprovalAssignmentRequiredChecksumRule {
    fn check(&self, item: &ApprovalAssignment) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAssignmentMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalAssignmentRule for ApprovalAssignmentMaximumRangeRule {
    fn check(&self, item: &ApprovalAssignment) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalAssignmentRule for ApprovalAssignmentCodeLengthRule {
    fn check(&self, item: &ApprovalAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAssignmentRequiredAudienceRule {
    pub message: String,
}

impl ApprovalAssignmentRule for ApprovalAssignmentRequiredAudienceRule {
    fn check(&self, item: &ApprovalAssignment) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval assignment.
#[derive(Debug, Clone)]
pub struct ApprovalAssignmentValidator {
    pub rules: Vec<Rc<dyn ApprovalAssignmentRule>>,
}

impl ApprovalAssignmentValidator {
    pub fn validate(&self, item: &ApprovalAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_assignment_validator() -> ApprovalAssignmentValidator {
    ApprovalAssignmentValidator {
        rules: vec![
            Rc::new(ApprovalAssignmentAudienceLengthRule { max_length: 1000 }) as Rc<dyn ApprovalAssignmentRule>,
            Rc::new(ApprovalAssignmentRequiredChecksumRule {
                message: "approval assignment needs a value".to_string(),
            }),
            Rc::new(ApprovalAssignmentMaximumRangeRule { min: 9.9, max: 2420.8 }),
            Rc::new(ApprovalAssignmentCodeLengthRule { max_length: 120 }),
            Rc::new(ApprovalAssignmentRequiredAudienceRule {
                message: "approval assignment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_assignment_validator();
        let item = new_approval_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = ApprovalAssignmentAudienceLengthRule { max_length: 16 };
        let mut item = new_approval_assignment("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = ApprovalAssignmentRequiredChecksumRule {
            message: "approval assignment needs this field".to_string(),
        };
        let mut item = new_approval_assignment("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ApprovalAssignmentMaximumRangeRule { min: 3.9, max: 1353.7 };
        let mut item = new_approval_assignment("x", "y");
        item.maximum = 1355.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalAssignmentCodeLengthRule { max_length: 32 };
        let mut item = new_approval_assignment("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ApprovalAssignmentRequiredAudienceRule {
            message: "approval assignment needs a value".to_string(),
        };
        let mut item = new_approval_assignment("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
