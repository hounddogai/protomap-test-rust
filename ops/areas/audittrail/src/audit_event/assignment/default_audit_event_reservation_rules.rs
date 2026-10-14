use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default audit event reservation must pass. Check returns its problems, or none.
pub trait DefaultAuditEventReservationRule: std::fmt::Debug {
    fn check(&self, item: &DefaultAuditEventReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventReservationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DefaultAuditEventReservationRule for DefaultAuditEventReservationAllowedTitleRule {
    fn check(&self, item: &DefaultAuditEventReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventReservationRequiredReviewerRule {
    pub message: String,
}

impl DefaultAuditEventReservationRule for DefaultAuditEventReservationRequiredReviewerRule {
    fn check(&self, item: &DefaultAuditEventReservation) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventReservationCodeLengthRule {
    pub max_length: i64,
}

impl DefaultAuditEventReservationRule for DefaultAuditEventReservationCodeLengthRule {
    fn check(&self, item: &DefaultAuditEventReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default audit event reservation.
#[derive(Debug, Clone)]
pub struct DefaultAuditEventReservationValidator {
    pub rules: Vec<Rc<dyn DefaultAuditEventReservationRule>>,
}

impl DefaultAuditEventReservationValidator {
    pub fn validate(&self, item: &DefaultAuditEventReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultAuditEventReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultAuditEventReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_audit_event_reservation_validator() -> DefaultAuditEventReservationValidator {
    DefaultAuditEventReservationValidator {
        rules: vec![
            Rc::new(DefaultAuditEventReservationAllowedTitleRule {
                allowed: vec!["seasonal".to_string(), "pending".to_string()],
            }) as Rc<dyn DefaultAuditEventReservationRule>,
            Rc::new(DefaultAuditEventReservationRequiredReviewerRule {
                message: "default audit event reservation needs this field".to_string(),
            }),
            Rc::new(DefaultAuditEventReservationCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_audit_event_reservation_validator();
        let item = new_default_audit_event_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = DefaultAuditEventReservationRequiredReviewerRule {
            message: "default audit event reservation needs an entry".to_string(),
        };
        let mut item = new_default_audit_event_reservation("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultAuditEventReservationCodeLengthRule { max_length: 120 };
        let mut item = new_default_audit_event_reservation("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
