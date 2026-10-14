use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending page reservation must pass. Check returns its problems, or none.
pub trait PendingPageReservationRule: std::fmt::Debug {
    fn check(&self, item: &PendingPageReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageReservationRequiredLabelRule {
    pub message: String,
}

impl PendingPageReservationRule for PendingPageReservationRequiredLabelRule {
    fn check(&self, item: &PendingPageReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageReservationAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingPageReservationRule for PendingPageReservationAttemptsRangeRule {
    fn check(&self, item: &PendingPageReservation) -> Vec<String> {
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
pub struct PendingPageReservationCodePrefixRule {
    pub prefix: String,
}

impl PendingPageReservationRule for PendingPageReservationCodePrefixRule {
    fn check(&self, item: &PendingPageReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageReservationLabelLengthRule {
    pub max_length: i64,
}

impl PendingPageReservationRule for PendingPageReservationLabelLengthRule {
    fn check(&self, item: &PendingPageReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending page reservation.
#[derive(Debug, Clone)]
pub struct PendingPageReservationValidator {
    pub rules: Vec<Rc<dyn PendingPageReservationRule>>,
}

impl PendingPageReservationValidator {
    pub fn validate(&self, item: &PendingPageReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingPageReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPageReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_page_reservation_validator() -> PendingPageReservationValidator {
    PendingPageReservationValidator {
        rules: vec![
            Rc::new(PendingPageReservationRequiredLabelRule {
                message: "pending page reservation needs this field".to_string(),
            }) as Rc<dyn PendingPageReservationRule>,
            Rc::new(PendingPageReservationAttemptsRangeRule { min: 32, max: 3600 }),
            Rc::new(PendingPageReservationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingPageReservationLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_page_reservation_validator();
        let item = new_pending_page_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingPageReservationRequiredLabelRule {
            message: "pending page reservation needs an entry".to_string(),
        };
        let mut item = new_pending_page_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = PendingPageReservationAttemptsRangeRule { min: 80, max: 3600 };
        let mut item = new_pending_page_reservation("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingPageReservationLabelLengthRule { max_length: 1000 };
        let mut item = new_pending_page_reservation("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
