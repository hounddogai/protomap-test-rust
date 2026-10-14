use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted slo reservation must pass. Check returns its problems, or none.
pub trait WeightedSloReservationRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSloReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSloReservationChannelLengthRule {
    pub max_length: i64,
}

impl WeightedSloReservationRule for WeightedSloReservationChannelLengthRule {
    fn check(&self, item: &WeightedSloReservation) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSloReservationWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedSloReservationRule for WeightedSloReservationWindowDaysRangeRule {
    fn check(&self, item: &WeightedSloReservation) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSloReservationTimezoneLengthRule {
    pub max_length: i64,
}

impl WeightedSloReservationRule for WeightedSloReservationTimezoneLengthRule {
    fn check(&self, item: &WeightedSloReservation) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted slo reservation.
#[derive(Debug, Clone)]
pub struct WeightedSloReservationValidator {
    pub rules: Vec<Rc<dyn WeightedSloReservationRule>>,
}

impl WeightedSloReservationValidator {
    pub fn validate(&self, item: &WeightedSloReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedSloReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSloReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_slo_reservation_validator() -> WeightedSloReservationValidator {
    WeightedSloReservationValidator {
        rules: vec![
            Rc::new(WeightedSloReservationChannelLengthRule { max_length: 86400 })
                as Rc<dyn WeightedSloReservationRule>,
            Rc::new(WeightedSloReservationWindowDaysRangeRule { min: 64, max: 255 }),
            Rc::new(WeightedSloReservationTimezoneLengthRule { max_length: 3600 }),
        ],
    }
}
