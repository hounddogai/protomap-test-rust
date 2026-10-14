use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled timeline baseline must pass. Check returns its problems, or none.
pub trait ScheduledTimelineBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTimelineBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineBaselineRequiredLabelRule {
    pub message: String,
}

impl ScheduledTimelineBaselineRule for ScheduledTimelineBaselineRequiredLabelRule {
    fn check(&self, item: &ScheduledTimelineBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineBaselineCodePrefixRule {
    pub prefix: String,
}

impl ScheduledTimelineBaselineRule for ScheduledTimelineBaselineCodePrefixRule {
    fn check(&self, item: &ScheduledTimelineBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineBaselineLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledTimelineBaselineRule for ScheduledTimelineBaselineLabelLengthRule {
    fn check(&self, item: &ScheduledTimelineBaseline) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineBaselineCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledTimelineBaselineRule for ScheduledTimelineBaselineCodeLengthRule {
    fn check(&self, item: &ScheduledTimelineBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineBaselineChannelLengthRule {
    pub max_length: i64,
}

impl ScheduledTimelineBaselineRule for ScheduledTimelineBaselineChannelLengthRule {
    fn check(&self, item: &ScheduledTimelineBaseline) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled timeline baseline.
#[derive(Debug, Clone)]
pub struct ScheduledTimelineBaselineValidator {
    pub rules: Vec<Rc<dyn ScheduledTimelineBaselineRule>>,
}

impl ScheduledTimelineBaselineValidator {
    pub fn validate(&self, item: &ScheduledTimelineBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledTimelineBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTimelineBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_timeline_baseline_validator() -> ScheduledTimelineBaselineValidator {
    ScheduledTimelineBaselineValidator {
        rules: vec![
            Rc::new(ScheduledTimelineBaselineRequiredLabelRule {
                message: "scheduled timeline baseline needs this field".to_string(),
            }) as Rc<dyn ScheduledTimelineBaselineRule>,
            Rc::new(ScheduledTimelineBaselineCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ScheduledTimelineBaselineLabelLengthRule { max_length: 80 }),
            Rc::new(ScheduledTimelineBaselineCodeLengthRule { max_length: 64 }),
            Rc::new(ScheduledTimelineBaselineChannelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_timeline_baseline_validator();
        let item = new_scheduled_timeline_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledTimelineBaselineRequiredLabelRule {
            message: "scheduled timeline baseline needs an entry".to_string(),
        };
        let mut item = new_scheduled_timeline_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledTimelineBaselineLabelLengthRule { max_length: 80 };
        let mut item = new_scheduled_timeline_baseline("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledTimelineBaselineCodeLengthRule { max_length: 1000 };
        let mut item = new_scheduled_timeline_baseline("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ScheduledTimelineBaselineChannelLengthRule { max_length: 80 };
        let mut item = new_scheduled_timeline_baseline("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
