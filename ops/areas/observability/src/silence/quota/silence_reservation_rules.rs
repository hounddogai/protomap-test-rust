use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence reservation must pass. Check returns its problems, or none.
pub trait SilenceReservationRule: std::fmt::Debug {
    fn check(&self, item: &SilenceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReservationRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceReservationRule for SilenceReservationRevisionRangeRule {
    fn check(&self, item: &SilenceReservation) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReservationRequiredCodeRule {
    pub message: String,
}

impl SilenceReservationRule for SilenceReservationRequiredCodeRule {
    fn check(&self, item: &SilenceReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReservationCipherLengthRule {
    pub max_length: i64,
}

impl SilenceReservationRule for SilenceReservationCipherLengthRule {
    fn check(&self, item: &SilenceReservation) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReservationNameLengthRule {
    pub max_length: i64,
}

impl SilenceReservationRule for SilenceReservationNameLengthRule {
    fn check(&self, item: &SilenceReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence reservation.
#[derive(Debug, Clone)]
pub struct SilenceReservationValidator {
    pub rules: Vec<Rc<dyn SilenceReservationRule>>,
}

impl SilenceReservationValidator {
    pub fn validate(&self, item: &SilenceReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_reservation_validator() -> SilenceReservationValidator {
    SilenceReservationValidator {
        rules: vec![
            Rc::new(SilenceReservationRevisionRangeRule { min: 32, max: 1000 }) as Rc<dyn SilenceReservationRule>,
            Rc::new(SilenceReservationRequiredCodeRule {
                message: "silence reservation needs this field".to_string(),
            }),
            Rc::new(SilenceReservationCipherLengthRule { max_length: 32 }),
            Rc::new(SilenceReservationNameLengthRule { max_length: 1000 }),
        ],
    }
}
