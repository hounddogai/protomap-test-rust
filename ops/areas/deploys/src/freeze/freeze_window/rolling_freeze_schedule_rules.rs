use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling freeze schedule must pass. Check returns its problems, or none.
pub trait RollingFreezeScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RollingFreezeSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFreezeScheduleCodeLengthRule {
    pub max_length: i64,
}

impl RollingFreezeScheduleRule for RollingFreezeScheduleCodeLengthRule {
    fn check(&self, item: &RollingFreezeSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFreezeScheduleAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl RollingFreezeScheduleRule for RollingFreezeScheduleAllowedCountryRule {
    fn check(&self, item: &RollingFreezeSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is stale", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFreezeScheduleRequiredCodeRule {
    pub message: String,
}

impl RollingFreezeScheduleRule for RollingFreezeScheduleRequiredCodeRule {
    fn check(&self, item: &RollingFreezeSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFreezeScheduleVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingFreezeScheduleRule for RollingFreezeScheduleVersionRangeRule {
    fn check(&self, item: &RollingFreezeSchedule) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFreezeScheduleRequiredLocaleRule {
    pub message: String,
}

impl RollingFreezeScheduleRule for RollingFreezeScheduleRequiredLocaleRule {
    fn check(&self, item: &RollingFreezeSchedule) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling freeze schedule.
#[derive(Debug, Clone)]
pub struct RollingFreezeScheduleValidator {
    pub rules: Vec<Rc<dyn RollingFreezeScheduleRule>>,
}

impl RollingFreezeScheduleValidator {
    pub fn validate(&self, item: &RollingFreezeSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingFreezeSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingFreezeSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_freeze_schedule_validator() -> RollingFreezeScheduleValidator {
    RollingFreezeScheduleValidator {
        rules: vec![
            Rc::new(RollingFreezeScheduleCodeLengthRule { max_length: 86400 }) as Rc<dyn RollingFreezeScheduleRule>,
            Rc::new(RollingFreezeScheduleAllowedCountryRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(RollingFreezeScheduleRequiredCodeRule {
                message: "rolling freeze schedule needs an entry".to_string(),
            }),
            Rc::new(RollingFreezeScheduleVersionRangeRule { min: 64, max: 3600 }),
            Rc::new(RollingFreezeScheduleRequiredLocaleRule {
                message: "rolling freeze schedule needs a value".to_string(),
            }),
        ],
    }
}
