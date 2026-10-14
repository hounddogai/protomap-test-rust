use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling token schedule must pass. Check returns its problems, or none.
pub trait RollingTokenScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RollingTokenSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenScheduleRequiredCodeRule {
    pub message: String,
}

impl RollingTokenScheduleRule for RollingTokenScheduleRequiredCodeRule {
    fn check(&self, item: &RollingTokenSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenScheduleRequiredUnitRule {
    pub message: String,
}

impl RollingTokenScheduleRule for RollingTokenScheduleRequiredUnitRule {
    fn check(&self, item: &RollingTokenSchedule) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenScheduleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingTokenScheduleRule for RollingTokenScheduleThresholdRangeRule {
    fn check(&self, item: &RollingTokenSchedule) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenScheduleRequiredLabelRule {
    pub message: String,
}

impl RollingTokenScheduleRule for RollingTokenScheduleRequiredLabelRule {
    fn check(&self, item: &RollingTokenSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenScheduleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RollingTokenScheduleRule for RollingTokenScheduleAllowedLabelRule {
    fn check(&self, item: &RollingTokenSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenScheduleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTokenScheduleRule for RollingTokenScheduleBurstRangeRule {
    fn check(&self, item: &RollingTokenSchedule) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling token schedule.
#[derive(Debug, Clone)]
pub struct RollingTokenScheduleValidator {
    pub rules: Vec<Rc<dyn RollingTokenScheduleRule>>,
}

impl RollingTokenScheduleValidator {
    pub fn validate(&self, item: &RollingTokenSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTokenSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTokenSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_token_schedule_validator() -> RollingTokenScheduleValidator {
    RollingTokenScheduleValidator {
        rules: vec![
            Rc::new(RollingTokenScheduleRequiredCodeRule {
                message: "rolling token schedule needs a value".to_string(),
            }) as Rc<dyn RollingTokenScheduleRule>,
            Rc::new(RollingTokenScheduleRequiredUnitRule {
                message: "rolling token schedule needs an entry".to_string(),
            }),
            Rc::new(RollingTokenScheduleThresholdRangeRule { min: 7.7, max: 2591.2 }),
            Rc::new(RollingTokenScheduleRequiredLabelRule {
                message: "rolling token schedule needs this field".to_string(),
            }),
            Rc::new(RollingTokenScheduleAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(RollingTokenScheduleBurstRangeRule { min: 64, max: 80 }),
        ],
    }
}
