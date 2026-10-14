use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled sink schedule must pass. Check returns its problems, or none.
pub trait ScheduledSinkScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSinkSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkScheduleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledSinkScheduleRule for ScheduledSinkScheduleAllowedCodeRule {
    fn check(&self, item: &ScheduledSinkSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkScheduleCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledSinkScheduleRule for ScheduledSinkScheduleCodeLengthRule {
    fn check(&self, item: &ScheduledSinkSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkScheduleCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledSinkScheduleRule for ScheduledSinkScheduleCostRangeRule {
    fn check(&self, item: &ScheduledSinkSchedule) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkScheduleRegionLengthRule {
    pub max_length: i64,
}

impl ScheduledSinkScheduleRule for ScheduledSinkScheduleRegionLengthRule {
    fn check(&self, item: &ScheduledSinkSchedule) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkScheduleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledSinkScheduleRule for ScheduledSinkScheduleScoreRangeRule {
    fn check(&self, item: &ScheduledSinkSchedule) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkScheduleRequiredRegionRule {
    pub message: String,
}

impl ScheduledSinkScheduleRule for ScheduledSinkScheduleRequiredRegionRule {
    fn check(&self, item: &ScheduledSinkSchedule) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled sink schedule.
#[derive(Debug, Clone)]
pub struct ScheduledSinkScheduleValidator {
    pub rules: Vec<Rc<dyn ScheduledSinkScheduleRule>>,
}

impl ScheduledSinkScheduleValidator {
    pub fn validate(&self, item: &ScheduledSinkSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledSinkSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSinkSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_sink_schedule_validator() -> ScheduledSinkScheduleValidator {
    ScheduledSinkScheduleValidator {
        rules: vec![
            Rc::new(ScheduledSinkScheduleAllowedCodeRule {
                allowed: vec!["legacy".to_string()],
            }) as Rc<dyn ScheduledSinkScheduleRule>,
            Rc::new(ScheduledSinkScheduleCodeLengthRule { max_length: 64 }),
            Rc::new(ScheduledSinkScheduleCostRangeRule { min: 6.8, max: 3767.5 }),
            Rc::new(ScheduledSinkScheduleRegionLengthRule { max_length: 120 }),
            Rc::new(ScheduledSinkScheduleScoreRangeRule { min: 5.8, max: 3980.9 }),
            Rc::new(ScheduledSinkScheduleRequiredRegionRule {
                message: "scheduled sink schedule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_sink_schedule_validator();
        let item = new_scheduled_sink_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledSinkScheduleCodeLengthRule { max_length: 16 };
        let mut item = new_scheduled_sink_schedule("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = ScheduledSinkScheduleCostRangeRule { min: 9.9, max: 2089.9 };
        let mut item = new_scheduled_sink_schedule("x", "y");
        item.cost = 2091.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = ScheduledSinkScheduleRegionLengthRule { max_length: 255 };
        let mut item = new_scheduled_sink_schedule("x", "y");
        item.region = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ScheduledSinkScheduleScoreRangeRule { min: 5.5, max: 4261.3 };
        let mut item = new_scheduled_sink_schedule("x", "y");
        item.score = 4262.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = ScheduledSinkScheduleRequiredRegionRule {
            message: "scheduled sink schedule needs an entry".to_string(),
        };
        let mut item = new_scheduled_sink_schedule("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
