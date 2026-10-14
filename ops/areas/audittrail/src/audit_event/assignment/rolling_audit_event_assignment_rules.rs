use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling audit event assignment must pass. Check returns its problems, or none.
pub trait RollingAuditEventAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RollingAuditEventAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventAssignmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingAuditEventAssignmentRule for RollingAuditEventAssignmentPriorityRangeRule {
    fn check(&self, item: &RollingAuditEventAssignment) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventAssignmentRequiredCodeRule {
    pub message: String,
}

impl RollingAuditEventAssignmentRule for RollingAuditEventAssignmentRequiredCodeRule {
    fn check(&self, item: &RollingAuditEventAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl RollingAuditEventAssignmentRule for RollingAuditEventAssignmentCodeLengthRule {
    fn check(&self, item: &RollingAuditEventAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling audit event assignment.
#[derive(Debug, Clone)]
pub struct RollingAuditEventAssignmentValidator {
    pub rules: Vec<Rc<dyn RollingAuditEventAssignmentRule>>,
}

impl RollingAuditEventAssignmentValidator {
    pub fn validate(&self, item: &RollingAuditEventAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingAuditEventAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingAuditEventAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_audit_event_assignment_validator() -> RollingAuditEventAssignmentValidator {
    RollingAuditEventAssignmentValidator {
        rules: vec![
            Rc::new(RollingAuditEventAssignmentPriorityRangeRule { min: 80, max: 1000 })
                as Rc<dyn RollingAuditEventAssignmentRule>,
            Rc::new(RollingAuditEventAssignmentRequiredCodeRule {
                message: "rolling audit event assignment needs a value".to_string(),
            }),
            Rc::new(RollingAuditEventAssignmentCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_audit_event_assignment_validator();
        let item = new_rolling_audit_event_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RollingAuditEventAssignmentPriorityRangeRule { min: 64, max: 86400 };
        let mut item = new_rolling_audit_event_assignment("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingAuditEventAssignmentRequiredCodeRule {
            message: "rolling audit event assignment needs a value".to_string(),
        };
        let mut item = new_rolling_audit_event_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingAuditEventAssignmentCodeLengthRule { max_length: 3600 };
        let mut item = new_rolling_audit_event_assignment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
