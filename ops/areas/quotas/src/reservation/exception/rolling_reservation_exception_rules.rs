use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling reservation exception must pass. Check returns its problems, or none.
pub trait RollingReservationExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RollingReservationException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationExceptionLocaleLengthRule {
    pub max_length: i64,
}

impl RollingReservationExceptionRule for RollingReservationExceptionLocaleLengthRule {
    fn check(&self, item: &RollingReservationException) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationExceptionRequiredChannelRule {
    pub message: String,
}

impl RollingReservationExceptionRule for RollingReservationExceptionRequiredChannelRule {
    fn check(&self, item: &RollingReservationException) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationExceptionRequiredLabelRule {
    pub message: String,
}

impl RollingReservationExceptionRule for RollingReservationExceptionRequiredLabelRule {
    fn check(&self, item: &RollingReservationException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationExceptionRequiredCodeRule {
    pub message: String,
}

impl RollingReservationExceptionRule for RollingReservationExceptionRequiredCodeRule {
    fn check(&self, item: &RollingReservationException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationExceptionLabelLengthRule {
    pub max_length: i64,
}

impl RollingReservationExceptionRule for RollingReservationExceptionLabelLengthRule {
    fn check(&self, item: &RollingReservationException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationExceptionRequiredLocaleRule {
    pub message: String,
}

impl RollingReservationExceptionRule for RollingReservationExceptionRequiredLocaleRule {
    fn check(&self, item: &RollingReservationException) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling reservation exception.
#[derive(Debug, Clone)]
pub struct RollingReservationExceptionValidator {
    pub rules: Vec<Rc<dyn RollingReservationExceptionRule>>,
}

impl RollingReservationExceptionValidator {
    pub fn validate(&self, item: &RollingReservationException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingReservationException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingReservationException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_reservation_exception_validator() -> RollingReservationExceptionValidator {
    RollingReservationExceptionValidator {
        rules: vec![
            Rc::new(RollingReservationExceptionLocaleLengthRule { max_length: 80 })
                as Rc<dyn RollingReservationExceptionRule>,
            Rc::new(RollingReservationExceptionRequiredChannelRule {
                message: "rolling reservation exception needs an entry".to_string(),
            }),
            Rc::new(RollingReservationExceptionRequiredLabelRule {
                message: "rolling reservation exception needs an entry".to_string(),
            }),
            Rc::new(RollingReservationExceptionRequiredCodeRule {
                message: "rolling reservation exception needs an entry".to_string(),
            }),
            Rc::new(RollingReservationExceptionLabelLengthRule { max_length: 80 }),
            Rc::new(RollingReservationExceptionRequiredLocaleRule {
                message: "rolling reservation exception needs a value".to_string(),
            }),
        ],
    }
}
