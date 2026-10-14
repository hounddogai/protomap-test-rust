use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending limit reservation must pass. Check returns its problems, or none.
pub trait PendingLimitReservationRule: std::fmt::Debug {
    fn check(&self, item: &PendingLimitReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitReservationNamespaceLengthRule {
    pub max_length: i64,
}

impl PendingLimitReservationRule for PendingLimitReservationNamespaceLengthRule {
    fn check(&self, item: &PendingLimitReservation) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitReservationCodeLengthRule {
    pub max_length: i64,
}

impl PendingLimitReservationRule for PendingLimitReservationCodeLengthRule {
    fn check(&self, item: &PendingLimitReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitReservationQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingLimitReservationRule for PendingLimitReservationQuotaUnitsRangeRule {
    fn check(&self, item: &PendingLimitReservation) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitReservationCodePrefixRule {
    pub prefix: String,
}

impl PendingLimitReservationRule for PendingLimitReservationCodePrefixRule {
    fn check(&self, item: &PendingLimitReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitReservationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingLimitReservationRule for PendingLimitReservationAllowedLabelRule {
    fn check(&self, item: &PendingLimitReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending limit reservation.
#[derive(Debug, Clone)]
pub struct PendingLimitReservationValidator {
    pub rules: Vec<Rc<dyn PendingLimitReservationRule>>,
}

impl PendingLimitReservationValidator {
    pub fn validate(&self, item: &PendingLimitReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingLimitReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingLimitReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_limit_reservation_validator() -> PendingLimitReservationValidator {
    PendingLimitReservationValidator {
        rules: vec![
            Rc::new(PendingLimitReservationNamespaceLengthRule { max_length: 16 })
                as Rc<dyn PendingLimitReservationRule>,
            Rc::new(PendingLimitReservationCodeLengthRule { max_length: 3600 }),
            Rc::new(PendingLimitReservationQuotaUnitsRangeRule { min: 1000, max: 86400 }),
            Rc::new(PendingLimitReservationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingLimitReservationAllowedLabelRule {
                allowed: vec!["batched".to_string()],
            }),
        ],
    }
}
