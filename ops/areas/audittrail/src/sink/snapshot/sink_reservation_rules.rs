use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink reservation must pass. Check returns its problems, or none.
pub trait SinkReservationRule: std::fmt::Debug {
    fn check(&self, item: &SinkReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReservationRequiredLabelRule {
    pub message: String,
}

impl SinkReservationRule for SinkReservationRequiredLabelRule {
    fn check(&self, item: &SinkReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReservationSourceLengthRule {
    pub max_length: i64,
}

impl SinkReservationRule for SinkReservationSourceLengthRule {
    fn check(&self, item: &SinkReservation) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReservationCurrencyLengthRule {
    pub max_length: i64,
}

impl SinkReservationRule for SinkReservationCurrencyLengthRule {
    fn check(&self, item: &SinkReservation) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReservationCodeLengthRule {
    pub max_length: i64,
}

impl SinkReservationRule for SinkReservationCodeLengthRule {
    fn check(&self, item: &SinkReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkReservationRequiredCurrencyRule {
    pub message: String,
}

impl SinkReservationRule for SinkReservationRequiredCurrencyRule {
    fn check(&self, item: &SinkReservation) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink reservation.
#[derive(Debug, Clone)]
pub struct SinkReservationValidator {
    pub rules: Vec<Rc<dyn SinkReservationRule>>,
}

impl SinkReservationValidator {
    pub fn validate(&self, item: &SinkReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_reservation_validator() -> SinkReservationValidator {
    SinkReservationValidator {
        rules: vec![
            Rc::new(SinkReservationRequiredLabelRule {
                message: "sink reservation needs this field".to_string(),
            }) as Rc<dyn SinkReservationRule>,
            Rc::new(SinkReservationSourceLengthRule { max_length: 80 }),
            Rc::new(SinkReservationCurrencyLengthRule { max_length: 32 }),
            Rc::new(SinkReservationCodeLengthRule { max_length: 3600 }),
            Rc::new(SinkReservationRequiredCurrencyRule {
                message: "sink reservation needs this field".to_string(),
            }),
        ],
    }
}
