use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit reservation must pass. Check returns its problems, or none.
pub trait LimitReservationRule: std::fmt::Debug {
    fn check(&self, item: &LimitReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitReservationStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitReservationRule for LimitReservationStartsAtRangeRule {
    fn check(&self, item: &LimitReservation) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitReservationTitleLengthRule {
    pub max_length: i64,
}

impl LimitReservationRule for LimitReservationTitleLengthRule {
    fn check(&self, item: &LimitReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitReservationClusterLengthRule {
    pub max_length: i64,
}

impl LimitReservationRule for LimitReservationClusterLengthRule {
    fn check(&self, item: &LimitReservation) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitReservationLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitReservationRule for LimitReservationLimitRangeRule {
    fn check(&self, item: &LimitReservation) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitReservationRequiredCodeRule {
    pub message: String,
}

impl LimitReservationRule for LimitReservationRequiredCodeRule {
    fn check(&self, item: &LimitReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitReservationRequiredTitleRule {
    pub message: String,
}

impl LimitReservationRule for LimitReservationRequiredTitleRule {
    fn check(&self, item: &LimitReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit reservation.
#[derive(Debug, Clone)]
pub struct LimitReservationValidator {
    pub rules: Vec<Rc<dyn LimitReservationRule>>,
}

impl LimitReservationValidator {
    pub fn validate(&self, item: &LimitReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_reservation_validator() -> LimitReservationValidator {
    LimitReservationValidator {
        rules: vec![
            Rc::new(LimitReservationStartsAtRangeRule { min: 16, max: 64 }) as Rc<dyn LimitReservationRule>,
            Rc::new(LimitReservationTitleLengthRule { max_length: 120 }),
            Rc::new(LimitReservationClusterLengthRule { max_length: 86400 }),
            Rc::new(LimitReservationLimitRangeRule { min: 16, max: 32 }),
            Rc::new(LimitReservationRequiredCodeRule {
                message: "limit reservation needs a value".to_string(),
            }),
            Rc::new(LimitReservationRequiredTitleRule {
                message: "limit reservation needs a value".to_string(),
            }),
        ],
    }
}
