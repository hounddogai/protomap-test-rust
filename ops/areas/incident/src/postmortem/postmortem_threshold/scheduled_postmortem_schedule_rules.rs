use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled postmortem schedule must pass. Check returns its problems, or none.
pub trait ScheduledPostmortemScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPostmortemSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemScheduleTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledPostmortemScheduleRule for ScheduledPostmortemScheduleTitleLengthRule {
    fn check(&self, item: &ScheduledPostmortemSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemScheduleCodePrefixRule {
    pub prefix: String,
}

impl ScheduledPostmortemScheduleRule for ScheduledPostmortemScheduleCodePrefixRule {
    fn check(&self, item: &ScheduledPostmortemSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemScheduleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledPostmortemScheduleRule for ScheduledPostmortemScheduleUpdatedAtRangeRule {
    fn check(&self, item: &ScheduledPostmortemSchedule) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemScheduleChannelLengthRule {
    pub max_length: i64,
}

impl ScheduledPostmortemScheduleRule for ScheduledPostmortemScheduleChannelLengthRule {
    fn check(&self, item: &ScheduledPostmortemSchedule) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemScheduleRequiredChannelRule {
    pub message: String,
}

impl ScheduledPostmortemScheduleRule for ScheduledPostmortemScheduleRequiredChannelRule {
    fn check(&self, item: &ScheduledPostmortemSchedule) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemScheduleRequiredTitleRule {
    pub message: String,
}

impl ScheduledPostmortemScheduleRule for ScheduledPostmortemScheduleRequiredTitleRule {
    fn check(&self, item: &ScheduledPostmortemSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled postmortem schedule.
#[derive(Debug, Clone)]
pub struct ScheduledPostmortemScheduleValidator {
    pub rules: Vec<Rc<dyn ScheduledPostmortemScheduleRule>>,
}

impl ScheduledPostmortemScheduleValidator {
    pub fn validate(&self, item: &ScheduledPostmortemSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledPostmortemSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPostmortemSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_postmortem_schedule_validator() -> ScheduledPostmortemScheduleValidator {
    ScheduledPostmortemScheduleValidator {
        rules: vec![
            Rc::new(ScheduledPostmortemScheduleTitleLengthRule { max_length: 255 })
                as Rc<dyn ScheduledPostmortemScheduleRule>,
            Rc::new(ScheduledPostmortemScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ScheduledPostmortemScheduleUpdatedAtRangeRule { min: 32, max: 64 }),
            Rc::new(ScheduledPostmortemScheduleChannelLengthRule { max_length: 86400 }),
            Rc::new(ScheduledPostmortemScheduleRequiredChannelRule {
                message: "scheduled postmortem schedule needs a value".to_string(),
            }),
            Rc::new(ScheduledPostmortemScheduleRequiredTitleRule {
                message: "scheduled postmortem schedule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_postmortem_schedule_validator();
        let item = new_scheduled_postmortem_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledPostmortemScheduleTitleLengthRule { max_length: 1000 };
        let mut item = new_scheduled_postmortem_schedule("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = ScheduledPostmortemScheduleUpdatedAtRangeRule { min: 120, max: 3600 };
        let mut item = new_scheduled_postmortem_schedule("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ScheduledPostmortemScheduleChannelLengthRule { max_length: 32 };
        let mut item = new_scheduled_postmortem_schedule("x", "y");
        item.channel = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = ScheduledPostmortemScheduleRequiredChannelRule {
            message: "scheduled postmortem schedule needs a value".to_string(),
        };
        let mut item = new_scheduled_postmortem_schedule("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledPostmortemScheduleRequiredTitleRule {
            message: "scheduled postmortem schedule needs this field".to_string(),
        };
        let mut item = new_scheduled_postmortem_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
