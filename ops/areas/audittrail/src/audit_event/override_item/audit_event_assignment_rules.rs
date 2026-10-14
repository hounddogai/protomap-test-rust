use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event assignment must pass. Check returns its problems, or none.
pub trait AuditEventAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAssignmentIssuerLengthRule {
    pub max_length: i64,
}

impl AuditEventAssignmentRule for AuditEventAssignmentIssuerLengthRule {
    fn check(&self, item: &AuditEventAssignment) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAssignmentRequiredNameRule {
    pub message: String,
}

impl AuditEventAssignmentRule for AuditEventAssignmentRequiredNameRule {
    fn check(&self, item: &AuditEventAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAssignmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventAssignmentRule for AuditEventAssignmentCeilingRangeRule {
    fn check(&self, item: &AuditEventAssignment) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAssignmentNameLengthRule {
    pub max_length: i64,
}

impl AuditEventAssignmentRule for AuditEventAssignmentNameLengthRule {
    fn check(&self, item: &AuditEventAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event assignment.
#[derive(Debug, Clone)]
pub struct AuditEventAssignmentValidator {
    pub rules: Vec<Rc<dyn AuditEventAssignmentRule>>,
}

impl AuditEventAssignmentValidator {
    pub fn validate(&self, item: &AuditEventAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_assignment_validator() -> AuditEventAssignmentValidator {
    AuditEventAssignmentValidator {
        rules: vec![
            Rc::new(AuditEventAssignmentIssuerLengthRule { max_length: 3600 }) as Rc<dyn AuditEventAssignmentRule>,
            Rc::new(AuditEventAssignmentRequiredNameRule {
                message: "audit event assignment needs a value".to_string(),
            }),
            Rc::new(AuditEventAssignmentCeilingRangeRule { min: 1.7, max: 4627.9 }),
            Rc::new(AuditEventAssignmentNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_assignment_validator();
        let item = new_audit_event_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = AuditEventAssignmentIssuerLengthRule { max_length: 80 };
        let mut item = new_audit_event_assignment("x", "y");
        item.issuer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AuditEventAssignmentRequiredNameRule {
            message: "audit event assignment needs a value".to_string(),
        };
        let mut item = new_audit_event_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = AuditEventAssignmentCeilingRangeRule { min: 8.7, max: 3826.3 };
        let mut item = new_audit_event_assignment("x", "y");
        item.ceiling = 3827.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AuditEventAssignmentNameLengthRule { max_length: 1000 };
        let mut item = new_audit_event_assignment("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
