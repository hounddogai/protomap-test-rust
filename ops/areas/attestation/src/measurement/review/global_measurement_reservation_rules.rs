use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global measurement reservation must pass. Check returns its problems, or none.
pub trait GlobalMeasurementReservationRule: std::fmt::Debug {
    fn check(&self, item: &GlobalMeasurementReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementReservationRequiredLabelRule {
    pub message: String,
}

impl GlobalMeasurementReservationRule for GlobalMeasurementReservationRequiredLabelRule {
    fn check(&self, item: &GlobalMeasurementReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementReservationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalMeasurementReservationRule for GlobalMeasurementReservationGraceDaysRangeRule {
    fn check(&self, item: &GlobalMeasurementReservation) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementReservationWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalMeasurementReservationRule for GlobalMeasurementReservationWeightRangeRule {
    fn check(&self, item: &GlobalMeasurementReservation) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementReservationCodeLengthRule {
    pub max_length: i64,
}

impl GlobalMeasurementReservationRule for GlobalMeasurementReservationCodeLengthRule {
    fn check(&self, item: &GlobalMeasurementReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global measurement reservation.
#[derive(Debug, Clone)]
pub struct GlobalMeasurementReservationValidator {
    pub rules: Vec<Rc<dyn GlobalMeasurementReservationRule>>,
}

impl GlobalMeasurementReservationValidator {
    pub fn validate(&self, item: &GlobalMeasurementReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalMeasurementReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalMeasurementReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_measurement_reservation_validator() -> GlobalMeasurementReservationValidator {
    GlobalMeasurementReservationValidator {
        rules: vec![
            Rc::new(GlobalMeasurementReservationRequiredLabelRule {
                message: "global measurement reservation needs an entry".to_string(),
            }) as Rc<dyn GlobalMeasurementReservationRule>,
            Rc::new(GlobalMeasurementReservationGraceDaysRangeRule { min: 64, max: 80 }),
            Rc::new(GlobalMeasurementReservationWeightRangeRule { min: 9.3, max: 1265.3 }),
            Rc::new(GlobalMeasurementReservationCodeLengthRule { max_length: 3600 }),
        ],
    }
}
