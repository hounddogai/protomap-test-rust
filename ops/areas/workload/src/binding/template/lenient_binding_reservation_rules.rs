use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient binding reservation must pass. Check returns its problems, or none.
pub trait LenientBindingReservationRule: std::fmt::Debug {
    fn check(&self, item: &LenientBindingReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingReservationRequiredNameRule {
    pub message: String,
}

impl LenientBindingReservationRule for LenientBindingReservationRequiredNameRule {
    fn check(&self, item: &LenientBindingReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingReservationQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientBindingReservationRule for LenientBindingReservationQuotaUnitsRangeRule {
    fn check(&self, item: &LenientBindingReservation) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingReservationSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientBindingReservationRule for LenientBindingReservationSequenceRangeRule {
    fn check(&self, item: &LenientBindingReservation) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient binding reservation.
#[derive(Debug, Clone)]
pub struct LenientBindingReservationValidator {
    pub rules: Vec<Rc<dyn LenientBindingReservationRule>>,
}

impl LenientBindingReservationValidator {
    pub fn validate(&self, item: &LenientBindingReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientBindingReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientBindingReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_binding_reservation_validator() -> LenientBindingReservationValidator {
    LenientBindingReservationValidator {
        rules: vec![
            Rc::new(LenientBindingReservationRequiredNameRule {
                message: "lenient binding reservation needs this field".to_string(),
            }) as Rc<dyn LenientBindingReservationRule>,
            Rc::new(LenientBindingReservationQuotaUnitsRangeRule { min: 1000, max: 3600 }),
            Rc::new(LenientBindingReservationSequenceRangeRule { min: 16, max: 1000 }),
        ],
    }
}
