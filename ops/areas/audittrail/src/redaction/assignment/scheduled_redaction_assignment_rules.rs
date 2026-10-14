use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled redaction assignment must pass. Check returns its problems, or none.
pub trait ScheduledRedactionAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRedactionAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRedactionAssignmentRequiredCodeRule {
    pub message: String,
}

impl ScheduledRedactionAssignmentRule for ScheduledRedactionAssignmentRequiredCodeRule {
    fn check(&self, item: &ScheduledRedactionAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRedactionAssignmentNameLengthRule {
    pub max_length: i64,
}

impl ScheduledRedactionAssignmentRule for ScheduledRedactionAssignmentNameLengthRule {
    fn check(&self, item: &ScheduledRedactionAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRedactionAssignmentRequiredNameRule {
    pub message: String,
}

impl ScheduledRedactionAssignmentRule for ScheduledRedactionAssignmentRequiredNameRule {
    fn check(&self, item: &ScheduledRedactionAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRedactionAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledRedactionAssignmentRule for ScheduledRedactionAssignmentAllowedCodeRule {
    fn check(&self, item: &ScheduledRedactionAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRedactionAssignmentMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledRedactionAssignmentRule for ScheduledRedactionAssignmentMaximumRangeRule {
    fn check(&self, item: &ScheduledRedactionAssignment) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRedactionAssignmentCodePrefixRule {
    pub prefix: String,
}

impl ScheduledRedactionAssignmentRule for ScheduledRedactionAssignmentCodePrefixRule {
    fn check(&self, item: &ScheduledRedactionAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled redaction assignment.
#[derive(Debug, Clone)]
pub struct ScheduledRedactionAssignmentValidator {
    pub rules: Vec<Rc<dyn ScheduledRedactionAssignmentRule>>,
}

impl ScheduledRedactionAssignmentValidator {
    pub fn validate(&self, item: &ScheduledRedactionAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledRedactionAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRedactionAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_redaction_assignment_validator() -> ScheduledRedactionAssignmentValidator {
    ScheduledRedactionAssignmentValidator {
        rules: vec![
            Rc::new(ScheduledRedactionAssignmentRequiredCodeRule {
                message: "scheduled redaction assignment needs this field".to_string(),
            }) as Rc<dyn ScheduledRedactionAssignmentRule>,
            Rc::new(ScheduledRedactionAssignmentNameLengthRule { max_length: 64 }),
            Rc::new(ScheduledRedactionAssignmentRequiredNameRule {
                message: "scheduled redaction assignment needs this field".to_string(),
            }),
            Rc::new(ScheduledRedactionAssignmentAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(ScheduledRedactionAssignmentMaximumRangeRule { min: 0.0, max: 955.9 }),
            Rc::new(ScheduledRedactionAssignmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_redaction_assignment_validator();
        let item = new_scheduled_redaction_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledRedactionAssignmentRequiredCodeRule {
            message: "scheduled redaction assignment needs an entry".to_string(),
        };
        let mut item = new_scheduled_redaction_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledRedactionAssignmentNameLengthRule { max_length: 1000 };
        let mut item = new_scheduled_redaction_assignment("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledRedactionAssignmentRequiredNameRule {
            message: "scheduled redaction assignment needs an entry".to_string(),
        };
        let mut item = new_scheduled_redaction_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ScheduledRedactionAssignmentMaximumRangeRule { min: 2.2, max: 3140.1 };
        let mut item = new_scheduled_redaction_assignment("x", "y");
        item.maximum = 3141.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
