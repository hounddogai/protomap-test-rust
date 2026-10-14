use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default reservation schedule must pass. Check returns its problems, or none.
pub trait DefaultReservationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultReservationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationScheduleAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl DefaultReservationScheduleRule for DefaultReservationScheduleAllowedUnitRule {
    fn check(&self, item: &DefaultReservationSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is not allowed", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSchedulePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultReservationScheduleRule for DefaultReservationSchedulePriorityRangeRule {
    fn check(&self, item: &DefaultReservationSchedule) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationScheduleCodeLengthRule {
    pub max_length: i64,
}

impl DefaultReservationScheduleRule for DefaultReservationScheduleCodeLengthRule {
    fn check(&self, item: &DefaultReservationSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationScheduleRequiredSummaryRule {
    pub message: String,
}

impl DefaultReservationScheduleRule for DefaultReservationScheduleRequiredSummaryRule {
    fn check(&self, item: &DefaultReservationSchedule) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationScheduleTitleLengthRule {
    pub max_length: i64,
}

impl DefaultReservationScheduleRule for DefaultReservationScheduleTitleLengthRule {
    fn check(&self, item: &DefaultReservationSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default reservation schedule.
#[derive(Debug, Clone)]
pub struct DefaultReservationScheduleValidator {
    pub rules: Vec<Rc<dyn DefaultReservationScheduleRule>>,
}

impl DefaultReservationScheduleValidator {
    pub fn validate(&self, item: &DefaultReservationSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultReservationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultReservationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_reservation_schedule_validator() -> DefaultReservationScheduleValidator {
    DefaultReservationScheduleValidator {
        rules: vec![
            Rc::new(DefaultReservationScheduleAllowedUnitRule {
                allowed: vec!["default".to_string()],
            }) as Rc<dyn DefaultReservationScheduleRule>,
            Rc::new(DefaultReservationSchedulePriorityRangeRule { min: 120, max: 86400 }),
            Rc::new(DefaultReservationScheduleCodeLengthRule { max_length: 1000 }),
            Rc::new(DefaultReservationScheduleRequiredSummaryRule {
                message: "default reservation schedule needs an entry".to_string(),
            }),
            Rc::new(DefaultReservationScheduleTitleLengthRule { max_length: 64 }),
        ],
    }
}
