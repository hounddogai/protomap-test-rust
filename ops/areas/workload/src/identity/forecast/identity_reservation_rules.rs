use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity reservation must pass. Check returns its problems, or none.
pub trait IdentityReservationRule: std::fmt::Debug {
    fn check(&self, item: &IdentityReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityReservationFingerprintLengthRule {
    pub max_length: i64,
}

impl IdentityReservationRule for IdentityReservationFingerprintLengthRule {
    fn check(&self, item: &IdentityReservation) -> Vec<String> {
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
pub struct IdentityReservationQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityReservationRule for IdentityReservationQuantityRangeRule {
    fn check(&self, item: &IdentityReservation) -> Vec<String> {
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
pub struct IdentityReservationCodePrefixRule {
    pub prefix: String,
}

impl IdentityReservationRule for IdentityReservationCodePrefixRule {
    fn check(&self, item: &IdentityReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityReservationCodeLengthRule {
    pub max_length: i64,
}

impl IdentityReservationRule for IdentityReservationCodeLengthRule {
    fn check(&self, item: &IdentityReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity reservation.
#[derive(Debug, Clone)]
pub struct IdentityReservationValidator {
    pub rules: Vec<Rc<dyn IdentityReservationRule>>,
}

impl IdentityReservationValidator {
    pub fn validate(&self, item: &IdentityReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_reservation_validator() -> IdentityReservationValidator {
    IdentityReservationValidator {
        rules: vec![
            Rc::new(IdentityReservationFingerprintLengthRule { max_length: 64 }) as Rc<dyn IdentityReservationRule>,
            Rc::new(IdentityReservationQuantityRangeRule { min: 32, max: 3600 }),
            Rc::new(IdentityReservationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(IdentityReservationCodeLengthRule { max_length: 80 }),
        ],
    }
}
