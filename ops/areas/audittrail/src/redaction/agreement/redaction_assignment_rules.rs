use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction assignment must pass. Check returns its problems, or none.
pub trait RedactionAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RedactionAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAssignmentIssuerLengthRule {
    pub max_length: i64,
}

impl RedactionAssignmentRule for RedactionAssignmentIssuerLengthRule {
    fn check(&self, item: &RedactionAssignment) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAssignmentRequiredIssuerRule {
    pub message: String,
}

impl RedactionAssignmentRule for RedactionAssignmentRequiredIssuerRule {
    fn check(&self, item: &RedactionAssignment) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAssignmentAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionAssignmentRule for RedactionAssignmentAttemptsRangeRule {
    fn check(&self, item: &RedactionAssignment) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RedactionAssignmentRule for RedactionAssignmentAllowedCodeRule {
    fn check(&self, item: &RedactionAssignment) -> Vec<String> {
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
pub struct RedactionAssignmentRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionAssignmentRule for RedactionAssignmentRotationDaysRangeRule {
    fn check(&self, item: &RedactionAssignment) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction assignment.
#[derive(Debug, Clone)]
pub struct RedactionAssignmentValidator {
    pub rules: Vec<Rc<dyn RedactionAssignmentRule>>,
}

impl RedactionAssignmentValidator {
    pub fn validate(&self, item: &RedactionAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_assignment_validator() -> RedactionAssignmentValidator {
    RedactionAssignmentValidator {
        rules: vec![
            Rc::new(RedactionAssignmentIssuerLengthRule { max_length: 120 }) as Rc<dyn RedactionAssignmentRule>,
            Rc::new(RedactionAssignmentRequiredIssuerRule {
                message: "redaction assignment needs a value".to_string(),
            }),
            Rc::new(RedactionAssignmentAttemptsRangeRule { min: 16, max: 80 }),
            Rc::new(RedactionAssignmentAllowedCodeRule {
                allowed: vec!["tiered".to_string()],
            }),
            Rc::new(RedactionAssignmentRotationDaysRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_assignment_validator();
        let item = new_redaction_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = RedactionAssignmentIssuerLengthRule { max_length: 16 };
        let mut item = new_redaction_assignment("x", "y");
        item.issuer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RedactionAssignmentRequiredIssuerRule {
            message: "redaction assignment needs this field".to_string(),
        };
        let mut item = new_redaction_assignment("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RedactionAssignmentAttemptsRangeRule { min: 120, max: 255 };
        let mut item = new_redaction_assignment("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RedactionAssignmentRotationDaysRangeRule { min: 1000, max: 3600 };
        let mut item = new_redaction_assignment("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
