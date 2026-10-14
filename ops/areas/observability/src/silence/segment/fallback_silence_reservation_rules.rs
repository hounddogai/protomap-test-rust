use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback silence reservation must pass. Check returns its problems, or none.
pub trait FallbackSilenceReservationRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSilenceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceReservationRequiredClusterRule {
    pub message: String,
}

impl FallbackSilenceReservationRule for FallbackSilenceReservationRequiredClusterRule {
    fn check(&self, item: &FallbackSilenceReservation) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceReservationRequiredCodeRule {
    pub message: String,
}

impl FallbackSilenceReservationRule for FallbackSilenceReservationRequiredCodeRule {
    fn check(&self, item: &FallbackSilenceReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceReservationCodePrefixRule {
    pub prefix: String,
}

impl FallbackSilenceReservationRule for FallbackSilenceReservationCodePrefixRule {
    fn check(&self, item: &FallbackSilenceReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceReservationRequiredTitleRule {
    pub message: String,
}

impl FallbackSilenceReservationRule for FallbackSilenceReservationRequiredTitleRule {
    fn check(&self, item: &FallbackSilenceReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceReservationCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackSilenceReservationRule for FallbackSilenceReservationCreatedAtRangeRule {
    fn check(&self, item: &FallbackSilenceReservation) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSilenceReservationTitleLengthRule {
    pub max_length: i64,
}

impl FallbackSilenceReservationRule for FallbackSilenceReservationTitleLengthRule {
    fn check(&self, item: &FallbackSilenceReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback silence reservation.
#[derive(Debug, Clone)]
pub struct FallbackSilenceReservationValidator {
    pub rules: Vec<Rc<dyn FallbackSilenceReservationRule>>,
}

impl FallbackSilenceReservationValidator {
    pub fn validate(&self, item: &FallbackSilenceReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackSilenceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSilenceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_silence_reservation_validator() -> FallbackSilenceReservationValidator {
    FallbackSilenceReservationValidator {
        rules: vec![
            Rc::new(FallbackSilenceReservationRequiredClusterRule {
                message: "fallback silence reservation needs a value".to_string(),
            }) as Rc<dyn FallbackSilenceReservationRule>,
            Rc::new(FallbackSilenceReservationRequiredCodeRule {
                message: "fallback silence reservation needs an entry".to_string(),
            }),
            Rc::new(FallbackSilenceReservationCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(FallbackSilenceReservationRequiredTitleRule {
                message: "fallback silence reservation needs a value".to_string(),
            }),
            Rc::new(FallbackSilenceReservationCreatedAtRangeRule { min: 16, max: 32 }),
            Rc::new(FallbackSilenceReservationTitleLengthRule { max_length: 3600 }),
        ],
    }
}
