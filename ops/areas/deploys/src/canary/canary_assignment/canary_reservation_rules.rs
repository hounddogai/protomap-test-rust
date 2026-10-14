use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary reservation must pass. Check returns its problems, or none.
pub trait CanaryReservationRule: std::fmt::Debug {
    fn check(&self, item: &CanaryReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryReservationLabelLengthRule {
    pub max_length: i64,
}

impl CanaryReservationRule for CanaryReservationLabelLengthRule {
    fn check(&self, item: &CanaryReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryReservationRequiredLabelRule {
    pub message: String,
}

impl CanaryReservationRule for CanaryReservationRequiredLabelRule {
    fn check(&self, item: &CanaryReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryReservationRequiredCodeRule {
    pub message: String,
}

impl CanaryReservationRule for CanaryReservationRequiredCodeRule {
    fn check(&self, item: &CanaryReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary reservation.
#[derive(Debug, Clone)]
pub struct CanaryReservationValidator {
    pub rules: Vec<Rc<dyn CanaryReservationRule>>,
}

impl CanaryReservationValidator {
    pub fn validate(&self, item: &CanaryReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_reservation_validator() -> CanaryReservationValidator {
    CanaryReservationValidator {
        rules: vec![
            Rc::new(CanaryReservationLabelLengthRule { max_length: 80 }) as Rc<dyn CanaryReservationRule>,
            Rc::new(CanaryReservationRequiredLabelRule {
                message: "canary reservation needs an entry".to_string(),
            }),
            Rc::new(CanaryReservationRequiredCodeRule {
                message: "canary reservation needs an entry".to_string(),
            }),
        ],
    }
}
