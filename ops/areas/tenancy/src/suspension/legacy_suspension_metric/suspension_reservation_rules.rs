use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension reservation must pass. Check returns its problems, or none.
pub trait SuspensionReservationRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReservationRequiredCodeRule {
    pub message: String,
}

impl SuspensionReservationRule for SuspensionReservationRequiredCodeRule {
    fn check(&self, item: &SuspensionReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReservationRequiredLabelRule {
    pub message: String,
}

impl SuspensionReservationRule for SuspensionReservationRequiredLabelRule {
    fn check(&self, item: &SuspensionReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReservationAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionReservationRule for SuspensionReservationAttemptsRangeRule {
    fn check(&self, item: &SuspensionReservation) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReservationRequiredPhoneRule {
    pub message: String,
}

impl SuspensionReservationRule for SuspensionReservationRequiredPhoneRule {
    fn check(&self, item: &SuspensionReservation) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReservationCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionReservationRule for SuspensionReservationCodeLengthRule {
    fn check(&self, item: &SuspensionReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension reservation.
#[derive(Debug, Clone)]
pub struct SuspensionReservationValidator {
    pub rules: Vec<Rc<dyn SuspensionReservationRule>>,
}

impl SuspensionReservationValidator {
    pub fn validate(&self, item: &SuspensionReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_reservation_validator() -> SuspensionReservationValidator {
    SuspensionReservationValidator {
        rules: vec![
            Rc::new(SuspensionReservationRequiredCodeRule {
                message: "suspension reservation needs an entry".to_string(),
            }) as Rc<dyn SuspensionReservationRule>,
            Rc::new(SuspensionReservationRequiredLabelRule {
                message: "suspension reservation needs this field".to_string(),
            }),
            Rc::new(SuspensionReservationAttemptsRangeRule { min: 80, max: 255 }),
            Rc::new(SuspensionReservationRequiredPhoneRule {
                message: "suspension reservation needs this field".to_string(),
            }),
            Rc::new(SuspensionReservationCodeLengthRule { max_length: 80 }),
        ],
    }
}
