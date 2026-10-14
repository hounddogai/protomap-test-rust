use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling release reservation must pass. Check returns its problems, or none.
pub trait RollingReleaseReservationRule: std::fmt::Debug {
    fn check(&self, item: &RollingReleaseReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseReservationMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingReleaseReservationRule for RollingReleaseReservationMaximumRangeRule {
    fn check(&self, item: &RollingReleaseReservation) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseReservationRequiredExternalRefRule {
    pub message: String,
}

impl RollingReleaseReservationRule for RollingReleaseReservationRequiredExternalRefRule {
    fn check(&self, item: &RollingReleaseReservation) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseReservationExternalRefLengthRule {
    pub max_length: i64,
}

impl RollingReleaseReservationRule for RollingReleaseReservationExternalRefLengthRule {
    fn check(&self, item: &RollingReleaseReservation) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseReservationRequiredCodeRule {
    pub message: String,
}

impl RollingReleaseReservationRule for RollingReleaseReservationRequiredCodeRule {
    fn check(&self, item: &RollingReleaseReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseReservationAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingReleaseReservationRule for RollingReleaseReservationAmountRangeRule {
    fn check(&self, item: &RollingReleaseReservation) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling release reservation.
#[derive(Debug, Clone)]
pub struct RollingReleaseReservationValidator {
    pub rules: Vec<Rc<dyn RollingReleaseReservationRule>>,
}

impl RollingReleaseReservationValidator {
    pub fn validate(&self, item: &RollingReleaseReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingReleaseReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingReleaseReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_release_reservation_validator() -> RollingReleaseReservationValidator {
    RollingReleaseReservationValidator {
        rules: vec![
            Rc::new(RollingReleaseReservationMaximumRangeRule { min: 1.5, max: 3645.6 })
                as Rc<dyn RollingReleaseReservationRule>,
            Rc::new(RollingReleaseReservationRequiredExternalRefRule {
                message: "rolling release reservation needs an entry".to_string(),
            }),
            Rc::new(RollingReleaseReservationExternalRefLengthRule { max_length: 86400 }),
            Rc::new(RollingReleaseReservationRequiredCodeRule {
                message: "rolling release reservation needs an entry".to_string(),
            }),
            Rc::new(RollingReleaseReservationAmountRangeRule { min: 3.3, max: 1529.4 }),
        ],
    }
}
