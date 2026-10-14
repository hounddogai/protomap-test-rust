use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped retention reservation must pass. Check returns its problems, or none.
pub trait CappedRetentionReservationRule: std::fmt::Debug {
    fn check(&self, item: &CappedRetentionReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionReservationScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedRetentionReservationRule for CappedRetentionReservationScoreRangeRule {
    fn check(&self, item: &CappedRetentionReservation) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionReservationNameLengthRule {
    pub max_length: i64,
}

impl CappedRetentionReservationRule for CappedRetentionReservationNameLengthRule {
    fn check(&self, item: &CappedRetentionReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionReservationRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedRetentionReservationRule for CappedRetentionReservationRateRangeRule {
    fn check(&self, item: &CappedRetentionReservation) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped retention reservation.
#[derive(Debug, Clone)]
pub struct CappedRetentionReservationValidator {
    pub rules: Vec<Rc<dyn CappedRetentionReservationRule>>,
}

impl CappedRetentionReservationValidator {
    pub fn validate(&self, item: &CappedRetentionReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRetentionReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRetentionReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_retention_reservation_validator() -> CappedRetentionReservationValidator {
    CappedRetentionReservationValidator {
        rules: vec![
            Rc::new(CappedRetentionReservationScoreRangeRule { min: 5.2, max: 2169.7 })
                as Rc<dyn CappedRetentionReservationRule>,
            Rc::new(CappedRetentionReservationNameLengthRule { max_length: 86400 }),
            Rc::new(CappedRetentionReservationRateRangeRule { min: 3.3, max: 3314.2 }),
        ],
    }
}
