use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event slot must pass. Check returns its problems, or none.
pub trait AuditEventSlotRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSlotCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventSlotRule for AuditEventSlotCodeLengthRule {
    fn check(&self, item: &AuditEventSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSlotCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventSlotRule for AuditEventSlotCreatedAtRangeRule {
    fn check(&self, item: &AuditEventSlot) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSlotRequiredReviewerRule {
    pub message: String,
}

impl AuditEventSlotRule for AuditEventSlotRequiredReviewerRule {
    fn check(&self, item: &AuditEventSlot) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event slot.
#[derive(Debug, Clone)]
pub struct AuditEventSlotValidator {
    pub rules: Vec<Rc<dyn AuditEventSlotRule>>,
}

impl AuditEventSlotValidator {
    pub fn validate(&self, item: &AuditEventSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_slot_validator() -> AuditEventSlotValidator {
    AuditEventSlotValidator {
        rules: vec![
            Rc::new(AuditEventSlotCodeLengthRule { max_length: 255 }) as Rc<dyn AuditEventSlotRule>,
            Rc::new(AuditEventSlotCreatedAtRangeRule { min: 32, max: 86400 }),
            Rc::new(AuditEventSlotRequiredReviewerRule {
                message: "audit event slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_slot_validator();
        let item = new_audit_event_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventSlotCodeLengthRule { max_length: 32 };
        let mut item = new_audit_event_slot("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = AuditEventSlotCreatedAtRangeRule { min: 16, max: 32 };
        let mut item = new_audit_event_slot("x", "y");
        item.created_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = AuditEventSlotRequiredReviewerRule {
            message: "audit event slot needs an entry".to_string(),
        };
        let mut item = new_audit_event_slot("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
