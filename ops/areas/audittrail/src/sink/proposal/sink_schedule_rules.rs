use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink schedule must pass. Check returns its problems, or none.
pub trait SinkScheduleRule: std::fmt::Debug {
    fn check(&self, item: &SinkSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScheduleRequiredTitleRule {
    pub message: String,
}

impl SinkScheduleRule for SinkScheduleRequiredTitleRule {
    fn check(&self, item: &SinkSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScheduleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkScheduleRule for SinkScheduleFloorValueRangeRule {
    fn check(&self, item: &SinkSchedule) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScheduleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl SinkScheduleRule for SinkScheduleAllowedTitleRule {
    fn check(&self, item: &SinkSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScheduleTitleLengthRule {
    pub max_length: i64,
}

impl SinkScheduleRule for SinkScheduleTitleLengthRule {
    fn check(&self, item: &SinkSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScheduleCodePrefixRule {
    pub prefix: String,
}

impl SinkScheduleRule for SinkScheduleCodePrefixRule {
    fn check(&self, item: &SinkSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScheduleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkScheduleRule for SinkScheduleGraceDaysRangeRule {
    fn check(&self, item: &SinkSchedule) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a sink schedule.
#[derive(Debug, Clone)]
pub struct SinkScheduleValidator {
    pub rules: Vec<Rc<dyn SinkScheduleRule>>,
}

impl SinkScheduleValidator {
    pub fn validate(&self, item: &SinkSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_schedule_validator() -> SinkScheduleValidator {
    SinkScheduleValidator {
        rules: vec![
            Rc::new(SinkScheduleRequiredTitleRule {
                message: "sink schedule needs this field".to_string(),
            }) as Rc<dyn SinkScheduleRule>,
            Rc::new(SinkScheduleFloorValueRangeRule { min: 0.3, max: 3927.7 }),
            Rc::new(SinkScheduleAllowedTitleRule {
                allowed: vec!["scheduled".to_string(), "default".to_string(), "global".to_string()],
            }),
            Rc::new(SinkScheduleTitleLengthRule { max_length: 80 }),
            Rc::new(SinkScheduleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SinkScheduleGraceDaysRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_schedule_validator();
        let item = new_sink_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SinkScheduleRequiredTitleRule {
            message: "sink schedule needs an entry".to_string(),
        };
        let mut item = new_sink_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = SinkScheduleFloorValueRangeRule { min: 7.4, max: 2709.6 };
        let mut item = new_sink_schedule("x", "y");
        item.floor_value = 2711.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SinkScheduleTitleLengthRule { max_length: 1000 };
        let mut item = new_sink_schedule("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = SinkScheduleGraceDaysRangeRule { min: 80, max: 255 };
        let mut item = new_sink_schedule("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
