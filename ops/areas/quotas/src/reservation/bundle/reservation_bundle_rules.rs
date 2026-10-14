use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation bundle must pass. Check returns its problems, or none.
pub trait ReservationBundleRule: std::fmt::Debug {
    fn check(&self, item: &ReservationBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBundleCipherLengthRule {
    pub max_length: i64,
}

impl ReservationBundleRule for ReservationBundleCipherLengthRule {
    fn check(&self, item: &ReservationBundle) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBundleLabelLengthRule {
    pub max_length: i64,
}

impl ReservationBundleRule for ReservationBundleLabelLengthRule {
    fn check(&self, item: &ReservationBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBundleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationBundleRule for ReservationBundleMaximumRangeRule {
    fn check(&self, item: &ReservationBundle) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBundleRequiredCipherRule {
    pub message: String,
}

impl ReservationBundleRule for ReservationBundleRequiredCipherRule {
    fn check(&self, item: &ReservationBundle) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation bundle.
#[derive(Debug, Clone)]
pub struct ReservationBundleValidator {
    pub rules: Vec<Rc<dyn ReservationBundleRule>>,
}

impl ReservationBundleValidator {
    pub fn validate(&self, item: &ReservationBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_bundle_validator() -> ReservationBundleValidator {
    ReservationBundleValidator {
        rules: vec![
            Rc::new(ReservationBundleCipherLengthRule { max_length: 32 }) as Rc<dyn ReservationBundleRule>,
            Rc::new(ReservationBundleLabelLengthRule { max_length: 3600 }),
            Rc::new(ReservationBundleMaximumRangeRule { min: 2.1, max: 2247.1 }),
            Rc::new(ReservationBundleRequiredCipherRule {
                message: "reservation bundle needs a value".to_string(),
            }),
        ],
    }
}
