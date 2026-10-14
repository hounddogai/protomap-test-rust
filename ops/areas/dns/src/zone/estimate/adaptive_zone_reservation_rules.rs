use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive zone reservation must pass. Check returns its problems, or none.
pub trait AdaptiveZoneReservationRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveZoneReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveZoneReservationRequiredLabelRule {
    pub message: String,
}

impl AdaptiveZoneReservationRule for AdaptiveZoneReservationRequiredLabelRule {
    fn check(&self, item: &AdaptiveZoneReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveZoneReservationCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveZoneReservationRule for AdaptiveZoneReservationCodePrefixRule {
    fn check(&self, item: &AdaptiveZoneReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveZoneReservationTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveZoneReservationRule for AdaptiveZoneReservationTtlSecondsRangeRule {
    fn check(&self, item: &AdaptiveZoneReservation) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveZoneReservationRequiredCodeRule {
    pub message: String,
}

impl AdaptiveZoneReservationRule for AdaptiveZoneReservationRequiredCodeRule {
    fn check(&self, item: &AdaptiveZoneReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveZoneReservationFingerprintLengthRule {
    pub max_length: i64,
}

impl AdaptiveZoneReservationRule for AdaptiveZoneReservationFingerprintLengthRule {
    fn check(&self, item: &AdaptiveZoneReservation) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveZoneReservationLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveZoneReservationRule for AdaptiveZoneReservationLabelLengthRule {
    fn check(&self, item: &AdaptiveZoneReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive zone reservation.
#[derive(Debug, Clone)]
pub struct AdaptiveZoneReservationValidator {
    pub rules: Vec<Rc<dyn AdaptiveZoneReservationRule>>,
}

impl AdaptiveZoneReservationValidator {
    pub fn validate(&self, item: &AdaptiveZoneReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveZoneReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveZoneReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_zone_reservation_validator() -> AdaptiveZoneReservationValidator {
    AdaptiveZoneReservationValidator {
        rules: vec![
            Rc::new(AdaptiveZoneReservationRequiredLabelRule {
                message: "adaptive zone reservation needs an entry".to_string(),
            }) as Rc<dyn AdaptiveZoneReservationRule>,
            Rc::new(AdaptiveZoneReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(AdaptiveZoneReservationTtlSecondsRangeRule { min: 255, max: 3600 }),
            Rc::new(AdaptiveZoneReservationRequiredCodeRule {
                message: "adaptive zone reservation needs a value".to_string(),
            }),
            Rc::new(AdaptiveZoneReservationFingerprintLengthRule { max_length: 255 }),
            Rc::new(AdaptiveZoneReservationLabelLengthRule { max_length: 32 }),
        ],
    }
}
