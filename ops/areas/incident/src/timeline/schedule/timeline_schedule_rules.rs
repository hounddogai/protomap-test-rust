use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline schedule must pass. Check returns its problems, or none.
pub trait TimelineScheduleRule: std::fmt::Debug {
    fn check(&self, item: &TimelineSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineScheduleChannelLengthRule {
    pub max_length: i64,
}

impl TimelineScheduleRule for TimelineScheduleChannelLengthRule {
    fn check(&self, item: &TimelineSchedule) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineScheduleAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl TimelineScheduleRule for TimelineScheduleAllowedKeyIdRule {
    fn check(&self, item: &TimelineSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is above maximum", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineScheduleCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineScheduleRule for TimelineScheduleCostRangeRule {
    fn check(&self, item: &TimelineSchedule) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline schedule.
#[derive(Debug, Clone)]
pub struct TimelineScheduleValidator {
    pub rules: Vec<Rc<dyn TimelineScheduleRule>>,
}

impl TimelineScheduleValidator {
    pub fn validate(&self, item: &TimelineSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_schedule_validator() -> TimelineScheduleValidator {
    TimelineScheduleValidator {
        rules: vec![
            Rc::new(TimelineScheduleChannelLengthRule { max_length: 80 }) as Rc<dyn TimelineScheduleRule>,
            Rc::new(TimelineScheduleAllowedKeyIdRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(TimelineScheduleCostRangeRule { min: 4.9, max: 1027.6 }),
        ],
    }
}
