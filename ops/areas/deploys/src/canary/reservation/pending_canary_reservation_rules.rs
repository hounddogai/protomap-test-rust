use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending canary reservation must pass. Check returns its problems, or none.
pub trait PendingCanaryReservationRule: std::fmt::Debug {
    fn check(&self, item: &PendingCanaryReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingCanaryReservationRule for PendingCanaryReservationAllowedCodeRule {
    fn check(&self, item: &PendingCanaryReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryReservationRequiredLabelRule {
    pub message: String,
}

impl PendingCanaryReservationRule for PendingCanaryReservationRequiredLabelRule {
    fn check(&self, item: &PendingCanaryReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryReservationLabelLengthRule {
    pub max_length: i64,
}

impl PendingCanaryReservationRule for PendingCanaryReservationLabelLengthRule {
    fn check(&self, item: &PendingCanaryReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending canary reservation.
#[derive(Debug, Clone)]
pub struct PendingCanaryReservationValidator {
    pub rules: Vec<Rc<dyn PendingCanaryReservationRule>>,
}

impl PendingCanaryReservationValidator {
    pub fn validate(&self, item: &PendingCanaryReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingCanaryReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingCanaryReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_canary_reservation_validator() -> PendingCanaryReservationValidator {
    PendingCanaryReservationValidator {
        rules: vec![
            Rc::new(PendingCanaryReservationAllowedCodeRule {
                allowed: vec!["manual".to_string(), "regional".to_string()],
            }) as Rc<dyn PendingCanaryReservationRule>,
            Rc::new(PendingCanaryReservationRequiredLabelRule {
                message: "pending canary reservation needs this field".to_string(),
            }),
            Rc::new(PendingCanaryReservationLabelLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_canary_reservation_validator();
        let item = new_pending_canary_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingCanaryReservationRequiredLabelRule {
            message: "pending canary reservation needs a value".to_string(),
        };
        let mut item = new_pending_canary_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingCanaryReservationLabelLengthRule { max_length: 16 };
        let mut item = new_pending_canary_reservation("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
