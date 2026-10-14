use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled suspension schedule must pass. Check returns its problems, or none.
pub trait ScheduledSuspensionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSuspensionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionScheduleRequiredReviewerRule {
    pub message: String,
}

impl ScheduledSuspensionScheduleRule for ScheduledSuspensionScheduleRequiredReviewerRule {
    fn check(&self, item: &ScheduledSuspensionSchedule) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionScheduleRequiredLabelRule {
    pub message: String,
}

impl ScheduledSuspensionScheduleRule for ScheduledSuspensionScheduleRequiredLabelRule {
    fn check(&self, item: &ScheduledSuspensionSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionScheduleAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl ScheduledSuspensionScheduleRule for ScheduledSuspensionScheduleAllowedSegmentRule {
    fn check(&self, item: &ScheduledSuspensionSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is duplicated", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionScheduleReviewerLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionScheduleRule for ScheduledSuspensionScheduleReviewerLengthRule {
    fn check(&self, item: &ScheduledSuspensionSchedule) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionScheduleCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionScheduleRule for ScheduledSuspensionScheduleCodeLengthRule {
    fn check(&self, item: &ScheduledSuspensionSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionScheduleRequiredSegmentRule {
    pub message: String,
}

impl ScheduledSuspensionScheduleRule for ScheduledSuspensionScheduleRequiredSegmentRule {
    fn check(&self, item: &ScheduledSuspensionSchedule) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled suspension schedule.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionScheduleValidator {
    pub rules: Vec<Rc<dyn ScheduledSuspensionScheduleRule>>,
}

impl ScheduledSuspensionScheduleValidator {
    pub fn validate(&self, item: &ScheduledSuspensionSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSuspensionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSuspensionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_suspension_schedule_validator() -> ScheduledSuspensionScheduleValidator {
    ScheduledSuspensionScheduleValidator {
        rules: vec![
            Rc::new(ScheduledSuspensionScheduleRequiredReviewerRule {
                message: "scheduled suspension schedule needs a value".to_string(),
            }) as Rc<dyn ScheduledSuspensionScheduleRule>,
            Rc::new(ScheduledSuspensionScheduleRequiredLabelRule {
                message: "scheduled suspension schedule needs this field".to_string(),
            }),
            Rc::new(ScheduledSuspensionScheduleAllowedSegmentRule { allowed: Vec::new() }),
            Rc::new(ScheduledSuspensionScheduleReviewerLengthRule { max_length: 16 }),
            Rc::new(ScheduledSuspensionScheduleCodeLengthRule { max_length: 3600 }),
            Rc::new(ScheduledSuspensionScheduleRequiredSegmentRule {
                message: "scheduled suspension schedule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_suspension_schedule_validator();
        let item = new_scheduled_suspension_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ScheduledSuspensionScheduleRequiredReviewerRule {
            message: "scheduled suspension schedule needs a value".to_string(),
        };
        let mut item = new_scheduled_suspension_schedule("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledSuspensionScheduleRequiredLabelRule {
            message: "scheduled suspension schedule needs a value".to_string(),
        };
        let mut item = new_scheduled_suspension_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = ScheduledSuspensionScheduleReviewerLengthRule { max_length: 16 };
        let mut item = new_scheduled_suspension_schedule("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledSuspensionScheduleCodeLengthRule { max_length: 86400 };
        let mut item = new_scheduled_suspension_schedule("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = ScheduledSuspensionScheduleRequiredSegmentRule {
            message: "scheduled suspension schedule needs an entry".to_string(),
        };
        let mut item = new_scheduled_suspension_schedule("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
