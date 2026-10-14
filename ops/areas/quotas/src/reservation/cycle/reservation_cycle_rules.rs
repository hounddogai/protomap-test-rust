use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a reservation cycle must pass. Check returns its problems, or none.
pub trait ReservationCycleRule: std::fmt::Debug {
    fn check(&self, item: &ReservationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationCycleRequiredCodeRule {
    pub message: String,
}

impl ReservationCycleRule for ReservationCycleRequiredCodeRule {
    fn check(&self, item: &ReservationCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationCycleAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl ReservationCycleRule for ReservationCycleAllowedChannelRule {
    fn check(&self, item: &ReservationCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is expired", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationCycleRequiredNameRule {
    pub message: String,
}

impl ReservationCycleRule for ReservationCycleRequiredNameRule {
    fn check(&self, item: &ReservationCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationCycleRequiredChannelRule {
    pub message: String,
}

impl ReservationCycleRule for ReservationCycleRequiredChannelRule {
    fn check(&self, item: &ReservationCycle) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation cycle.
#[derive(Debug, Clone)]
pub struct ReservationCycleValidator {
    pub rules: Vec<Rc<dyn ReservationCycleRule>>,
}

impl ReservationCycleValidator {
    pub fn validate(&self, item: &ReservationCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_cycle_validator() -> ReservationCycleValidator {
    ReservationCycleValidator {
        rules: vec![
            Rc::new(ReservationCycleRequiredCodeRule {
                message: "reservation cycle needs a value".to_string(),
            }) as Rc<dyn ReservationCycleRule>,
            Rc::new(ReservationCycleAllowedChannelRule {
                allowed: vec!["capped".to_string(), "tiered".to_string(), "lenient".to_string()],
            }),
            Rc::new(ReservationCycleRequiredNameRule {
                message: "reservation cycle needs this field".to_string(),
            }),
            Rc::new(ReservationCycleRequiredChannelRule {
                message: "reservation cycle needs an entry".to_string(),
            }),
        ],
    }
}
