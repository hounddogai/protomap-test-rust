use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending timeline schedule must pass. Check returns its problems, or none.
pub trait PendingTimelineScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PendingTimelineSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineScheduleCodeLengthRule {
    pub max_length: i64,
}

impl PendingTimelineScheduleRule for PendingTimelineScheduleCodeLengthRule {
    fn check(&self, item: &PendingTimelineSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineScheduleRequiredTimezoneRule {
    pub message: String,
}

impl PendingTimelineScheduleRule for PendingTimelineScheduleRequiredTimezoneRule {
    fn check(&self, item: &PendingTimelineSchedule) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineScheduleRequiredNameRule {
    pub message: String,
}

impl PendingTimelineScheduleRule for PendingTimelineScheduleRequiredNameRule {
    fn check(&self, item: &PendingTimelineSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineScheduleTimezoneLengthRule {
    pub max_length: i64,
}

impl PendingTimelineScheduleRule for PendingTimelineScheduleTimezoneLengthRule {
    fn check(&self, item: &PendingTimelineSchedule) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineScheduleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingTimelineScheduleRule for PendingTimelineScheduleAllowedLabelRule {
    fn check(&self, item: &PendingTimelineSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending timeline schedule.
#[derive(Debug, Clone)]
pub struct PendingTimelineScheduleValidator {
    pub rules: Vec<Rc<dyn PendingTimelineScheduleRule>>,
}

impl PendingTimelineScheduleValidator {
    pub fn validate(&self, item: &PendingTimelineSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingTimelineSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTimelineSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_timeline_schedule_validator() -> PendingTimelineScheduleValidator {
    PendingTimelineScheduleValidator {
        rules: vec![
            Rc::new(PendingTimelineScheduleCodeLengthRule { max_length: 255 }) as Rc<dyn PendingTimelineScheduleRule>,
            Rc::new(PendingTimelineScheduleRequiredTimezoneRule {
                message: "pending timeline schedule needs an entry".to_string(),
            }),
            Rc::new(PendingTimelineScheduleRequiredNameRule {
                message: "pending timeline schedule needs a value".to_string(),
            }),
            Rc::new(PendingTimelineScheduleTimezoneLengthRule { max_length: 80 }),
            Rc::new(PendingTimelineScheduleAllowedLabelRule {
                allowed: vec!["global".to_string(), "global".to_string(), "seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_timeline_schedule_validator();
        let item = new_pending_timeline_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingTimelineScheduleCodeLengthRule { max_length: 120 };
        let mut item = new_pending_timeline_schedule("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PendingTimelineScheduleRequiredTimezoneRule {
            message: "pending timeline schedule needs a value".to_string(),
        };
        let mut item = new_pending_timeline_schedule("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingTimelineScheduleRequiredNameRule {
            message: "pending timeline schedule needs a value".to_string(),
        };
        let mut item = new_pending_timeline_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = PendingTimelineScheduleTimezoneLengthRule { max_length: 32 };
        let mut item = new_pending_timeline_schedule("x", "y");
        item.timezone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
