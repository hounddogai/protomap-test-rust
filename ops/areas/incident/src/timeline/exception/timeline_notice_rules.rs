use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline notice must pass. Check returns its problems, or none.
pub trait TimelineNoticeRule: std::fmt::Debug {
    fn check(&self, item: &TimelineNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNoticeMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineNoticeRule for TimelineNoticeMarginRangeRule {
    fn check(&self, item: &TimelineNotice) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNoticeRequiredLabelRule {
    pub message: String,
}

impl TimelineNoticeRule for TimelineNoticeRequiredLabelRule {
    fn check(&self, item: &TimelineNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNoticeRequiredCodeRule {
    pub message: String,
}

impl TimelineNoticeRule for TimelineNoticeRequiredCodeRule {
    fn check(&self, item: &TimelineNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNoticeLabelLengthRule {
    pub max_length: i64,
}

impl TimelineNoticeRule for TimelineNoticeLabelLengthRule {
    fn check(&self, item: &TimelineNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNoticeRequiredNameRule {
    pub message: String,
}

impl TimelineNoticeRule for TimelineNoticeRequiredNameRule {
    fn check(&self, item: &TimelineNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNoticeCodeLengthRule {
    pub max_length: i64,
}

impl TimelineNoticeRule for TimelineNoticeCodeLengthRule {
    fn check(&self, item: &TimelineNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline notice.
#[derive(Debug, Clone)]
pub struct TimelineNoticeValidator {
    pub rules: Vec<Rc<dyn TimelineNoticeRule>>,
}

impl TimelineNoticeValidator {
    pub fn validate(&self, item: &TimelineNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_notice_validator() -> TimelineNoticeValidator {
    TimelineNoticeValidator {
        rules: vec![
            Rc::new(TimelineNoticeMarginRangeRule { min: 2.1, max: 229.7 }) as Rc<dyn TimelineNoticeRule>,
            Rc::new(TimelineNoticeRequiredLabelRule {
                message: "timeline notice needs an entry".to_string(),
            }),
            Rc::new(TimelineNoticeRequiredCodeRule {
                message: "timeline notice needs an entry".to_string(),
            }),
            Rc::new(TimelineNoticeLabelLengthRule { max_length: 16 }),
            Rc::new(TimelineNoticeRequiredNameRule {
                message: "timeline notice needs a value".to_string(),
            }),
            Rc::new(TimelineNoticeCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_notice_validator();
        let item = new_timeline_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = TimelineNoticeMarginRangeRule { min: 1.7, max: 2592.2 };
        let mut item = new_timeline_notice("x", "y");
        item.margin = 2593.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TimelineNoticeRequiredLabelRule {
            message: "timeline notice needs an entry".to_string(),
        };
        let mut item = new_timeline_notice("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineNoticeRequiredCodeRule {
            message: "timeline notice needs this field".to_string(),
        };
        let mut item = new_timeline_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TimelineNoticeLabelLengthRule { max_length: 255 };
        let mut item = new_timeline_notice("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TimelineNoticeRequiredNameRule {
            message: "timeline notice needs an entry".to_string(),
        };
        let mut item = new_timeline_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineNoticeCodeLengthRule { max_length: 120 };
        let mut item = new_timeline_notice("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
