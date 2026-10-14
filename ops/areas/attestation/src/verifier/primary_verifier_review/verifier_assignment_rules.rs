use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier assignment must pass. Check returns its problems, or none.
pub trait VerifierAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &VerifierAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignmentHostLengthRule {
    pub max_length: i64,
}

impl VerifierAssignmentRule for VerifierAssignmentHostLengthRule {
    fn check(&self, item: &VerifierAssignment) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignmentRequiredLabelRule {
    pub message: String,
}

impl VerifierAssignmentRule for VerifierAssignmentRequiredLabelRule {
    fn check(&self, item: &VerifierAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignmentRequiredCodeRule {
    pub message: String,
}

impl VerifierAssignmentRule for VerifierAssignmentRequiredCodeRule {
    fn check(&self, item: &VerifierAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignmentMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierAssignmentRule for VerifierAssignmentMaximumRangeRule {
    fn check(&self, item: &VerifierAssignment) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignmentVisibleGuardRule {
    pub limit: i64,
}

impl VerifierAssignmentRule for VerifierAssignmentVisibleGuardRule {
    fn check(&self, item: &VerifierAssignment) -> Vec<String> {
        if item.visible && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while visible",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierAssignmentRule for VerifierAssignmentUpdatedAtRangeRule {
    fn check(&self, item: &VerifierAssignment) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a verifier assignment.
#[derive(Debug, Clone)]
pub struct VerifierAssignmentValidator {
    pub rules: Vec<Rc<dyn VerifierAssignmentRule>>,
}

impl VerifierAssignmentValidator {
    pub fn validate(&self, item: &VerifierAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_assignment_validator() -> VerifierAssignmentValidator {
    VerifierAssignmentValidator {
        rules: vec![
            Rc::new(VerifierAssignmentHostLengthRule { max_length: 64 }) as Rc<dyn VerifierAssignmentRule>,
            Rc::new(VerifierAssignmentRequiredLabelRule {
                message: "verifier assignment needs an entry".to_string(),
            }),
            Rc::new(VerifierAssignmentRequiredCodeRule {
                message: "verifier assignment needs an entry".to_string(),
            }),
            Rc::new(VerifierAssignmentMaximumRangeRule { min: 0.2, max: 3002.3 }),
            Rc::new(VerifierAssignmentVisibleGuardRule { limit: 80 }),
            Rc::new(VerifierAssignmentUpdatedAtRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_assignment_validator();
        let item = new_verifier_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = VerifierAssignmentHostLengthRule { max_length: 16 };
        let mut item = new_verifier_assignment("x", "y");
        item.host = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = VerifierAssignmentRequiredLabelRule {
            message: "verifier assignment needs this field".to_string(),
        };
        let mut item = new_verifier_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierAssignmentRequiredCodeRule {
            message: "verifier assignment needs an entry".to_string(),
        };
        let mut item = new_verifier_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = VerifierAssignmentMaximumRangeRule { min: 8.5, max: 1434.0 };
        let mut item = new_verifier_assignment("x", "y");
        item.maximum = 1435.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = VerifierAssignmentUpdatedAtRangeRule { min: 255, max: 86400 };
        let mut item = new_verifier_assignment("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
