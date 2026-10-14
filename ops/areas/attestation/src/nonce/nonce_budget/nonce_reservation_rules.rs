use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a nonce reservation must pass. Check returns its problems, or none.
pub trait NonceReservationRule: std::fmt::Debug {
    fn check(&self, item: &NonceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceReservationPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceReservationRule for NonceReservationPriorityRangeRule {
    fn check(&self, item: &NonceReservation) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceReservationRequiredCipherRule {
    pub message: String,
}

impl NonceReservationRule for NonceReservationRequiredCipherRule {
    fn check(&self, item: &NonceReservation) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceReservationNameLengthRule {
    pub max_length: i64,
}

impl NonceReservationRule for NonceReservationNameLengthRule {
    fn check(&self, item: &NonceReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceReservationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl NonceReservationRule for NonceReservationAllowedNameRule {
    fn check(&self, item: &NonceReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceReservationCodePrefixRule {
    pub prefix: String,
}

impl NonceReservationRule for NonceReservationCodePrefixRule {
    fn check(&self, item: &NonceReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce reservation.
#[derive(Debug, Clone)]
pub struct NonceReservationValidator {
    pub rules: Vec<Rc<dyn NonceReservationRule>>,
}

impl NonceReservationValidator {
    pub fn validate(&self, item: &NonceReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_reservation_validator() -> NonceReservationValidator {
    NonceReservationValidator {
        rules: vec![
            Rc::new(NonceReservationPriorityRangeRule { min: 255, max: 3600 }) as Rc<dyn NonceReservationRule>,
            Rc::new(NonceReservationRequiredCipherRule {
                message: "nonce reservation needs a value".to_string(),
            }),
            Rc::new(NonceReservationNameLengthRule { max_length: 120 }),
            Rc::new(NonceReservationAllowedNameRule {
                allowed: vec!["capped".to_string(), "pending".to_string()],
            }),
            Rc::new(NonceReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
