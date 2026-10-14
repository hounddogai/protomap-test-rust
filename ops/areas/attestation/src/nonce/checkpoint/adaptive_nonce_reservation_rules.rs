use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive nonce reservation must pass. Check returns its problems, or none.
pub trait AdaptiveNonceReservationRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveNonceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceReservationNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveNonceReservationRule for AdaptiveNonceReservationNameLengthRule {
    fn check(&self, item: &AdaptiveNonceReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceReservationQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveNonceReservationRule for AdaptiveNonceReservationQuantityRangeRule {
    fn check(&self, item: &AdaptiveNonceReservation) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceReservationCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveNonceReservationRule for AdaptiveNonceReservationCodePrefixRule {
    fn check(&self, item: &AdaptiveNonceReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceReservationRequiredCodeRule {
    pub message: String,
}

impl AdaptiveNonceReservationRule for AdaptiveNonceReservationRequiredCodeRule {
    fn check(&self, item: &AdaptiveNonceReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceReservationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AdaptiveNonceReservationRule for AdaptiveNonceReservationAllowedNameRule {
    fn check(&self, item: &AdaptiveNonceReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive nonce reservation.
#[derive(Debug, Clone)]
pub struct AdaptiveNonceReservationValidator {
    pub rules: Vec<Rc<dyn AdaptiveNonceReservationRule>>,
}

impl AdaptiveNonceReservationValidator {
    pub fn validate(&self, item: &AdaptiveNonceReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveNonceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveNonceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_nonce_reservation_validator() -> AdaptiveNonceReservationValidator {
    AdaptiveNonceReservationValidator {
        rules: vec![
            Rc::new(AdaptiveNonceReservationNameLengthRule { max_length: 80 }) as Rc<dyn AdaptiveNonceReservationRule>,
            Rc::new(AdaptiveNonceReservationQuantityRangeRule { min: 16, max: 1000 }),
            Rc::new(AdaptiveNonceReservationCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(AdaptiveNonceReservationRequiredCodeRule {
                message: "adaptive nonce reservation needs this field".to_string(),
            }),
            Rc::new(AdaptiveNonceReservationAllowedNameRule {
                allowed: vec!["batched".to_string(), "batched".to_string(), "fallback".to_string()],
            }),
        ],
    }
}
