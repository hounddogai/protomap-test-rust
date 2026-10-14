use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict grant reservation must pass. Check returns its problems, or none.
pub trait StrictGrantReservationRule: std::fmt::Debug {
    fn check(&self, item: &StrictGrantReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantReservationScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictGrantReservationRule for StrictGrantReservationScoreRangeRule {
    fn check(&self, item: &StrictGrantReservation) -> Vec<String> {
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
pub struct StrictGrantReservationChannelLengthRule {
    pub max_length: i64,
}

impl StrictGrantReservationRule for StrictGrantReservationChannelLengthRule {
    fn check(&self, item: &StrictGrantReservation) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantReservationLocaleLengthRule {
    pub max_length: i64,
}

impl StrictGrantReservationRule for StrictGrantReservationLocaleLengthRule {
    fn check(&self, item: &StrictGrantReservation) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantReservationRequiredLocaleRule {
    pub message: String,
}

impl StrictGrantReservationRule for StrictGrantReservationRequiredLocaleRule {
    fn check(&self, item: &StrictGrantReservation) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantReservationRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictGrantReservationRule for StrictGrantReservationRateRangeRule {
    fn check(&self, item: &StrictGrantReservation) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantReservationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl StrictGrantReservationRule for StrictGrantReservationAllowedNameRule {
    fn check(&self, item: &StrictGrantReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict grant reservation.
#[derive(Debug, Clone)]
pub struct StrictGrantReservationValidator {
    pub rules: Vec<Rc<dyn StrictGrantReservationRule>>,
}

impl StrictGrantReservationValidator {
    pub fn validate(&self, item: &StrictGrantReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictGrantReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictGrantReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_grant_reservation_validator() -> StrictGrantReservationValidator {
    StrictGrantReservationValidator {
        rules: vec![
            Rc::new(StrictGrantReservationScoreRangeRule { min: 3.5, max: 3767.1 })
                as Rc<dyn StrictGrantReservationRule>,
            Rc::new(StrictGrantReservationChannelLengthRule { max_length: 86400 }),
            Rc::new(StrictGrantReservationLocaleLengthRule { max_length: 120 }),
            Rc::new(StrictGrantReservationRequiredLocaleRule {
                message: "strict grant reservation needs an entry".to_string(),
            }),
            Rc::new(StrictGrantReservationRateRangeRule { min: 4.6, max: 899.3 }),
            Rc::new(StrictGrantReservationAllowedNameRule {
                allowed: vec!["rolling".to_string(), "pending".to_string()],
            }),
        ],
    }
}
