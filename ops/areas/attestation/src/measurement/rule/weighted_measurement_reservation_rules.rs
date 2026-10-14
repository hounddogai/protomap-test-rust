use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted measurement reservation must pass. Check returns its problems, or none.
pub trait WeightedMeasurementReservationRule: std::fmt::Debug {
    fn check(&self, item: &WeightedMeasurementReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementReservationKeyIdLengthRule {
    pub max_length: i64,
}

impl WeightedMeasurementReservationRule for WeightedMeasurementReservationKeyIdLengthRule {
    fn check(&self, item: &WeightedMeasurementReservation) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementReservationTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedMeasurementReservationRule for WeightedMeasurementReservationTtlSecondsRangeRule {
    fn check(&self, item: &WeightedMeasurementReservation) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementReservationAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl WeightedMeasurementReservationRule for WeightedMeasurementReservationAllowedCountryRule {
    fn check(&self, item: &WeightedMeasurementReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is not allowed", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementReservationBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedMeasurementReservationRule for WeightedMeasurementReservationBatchSizeRangeRule {
    fn check(&self, item: &WeightedMeasurementReservation) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementReservationRequiredTitleRule {
    pub message: String,
}

impl WeightedMeasurementReservationRule for WeightedMeasurementReservationRequiredTitleRule {
    fn check(&self, item: &WeightedMeasurementReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted measurement reservation.
#[derive(Debug, Clone)]
pub struct WeightedMeasurementReservationValidator {
    pub rules: Vec<Rc<dyn WeightedMeasurementReservationRule>>,
}

impl WeightedMeasurementReservationValidator {
    pub fn validate(&self, item: &WeightedMeasurementReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedMeasurementReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedMeasurementReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_measurement_reservation_validator() -> WeightedMeasurementReservationValidator {
    WeightedMeasurementReservationValidator {
        rules: vec![
            Rc::new(WeightedMeasurementReservationKeyIdLengthRule { max_length: 16 })
                as Rc<dyn WeightedMeasurementReservationRule>,
            Rc::new(WeightedMeasurementReservationTtlSecondsRangeRule { min: 32, max: 32 }),
            Rc::new(WeightedMeasurementReservationAllowedCountryRule { allowed: Vec::new() }),
            Rc::new(WeightedMeasurementReservationBatchSizeRangeRule { min: 120, max: 86400 }),
            Rc::new(WeightedMeasurementReservationRequiredTitleRule {
                message: "weighted measurement reservation needs a value".to_string(),
            }),
        ],
    }
}
