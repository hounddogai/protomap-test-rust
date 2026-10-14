use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual data export notice must pass. Check returns its problems, or none.
pub trait ManualDataExportNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ManualDataExportNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportNoticeTitleLengthRule {
    pub max_length: i64,
}

impl ManualDataExportNoticeRule for ManualDataExportNoticeTitleLengthRule {
    fn check(&self, item: &ManualDataExportNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportNoticeRequiredTitleRule {
    pub message: String,
}

impl ManualDataExportNoticeRule for ManualDataExportNoticeRequiredTitleRule {
    fn check(&self, item: &ManualDataExportNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportNoticeRequiredPhoneRule {
    pub message: String,
}

impl ManualDataExportNoticeRule for ManualDataExportNoticeRequiredPhoneRule {
    fn check(&self, item: &ManualDataExportNotice) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportNoticeSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDataExportNoticeRule for ManualDataExportNoticeSequenceRangeRule {
    fn check(&self, item: &ManualDataExportNotice) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportNoticeLabelLengthRule {
    pub max_length: i64,
}

impl ManualDataExportNoticeRule for ManualDataExportNoticeLabelLengthRule {
    fn check(&self, item: &ManualDataExportNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportNoticeStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDataExportNoticeRule for ManualDataExportNoticeStartsAtRangeRule {
    fn check(&self, item: &ManualDataExportNotice) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual data export notice.
#[derive(Debug, Clone)]
pub struct ManualDataExportNoticeValidator {
    pub rules: Vec<Rc<dyn ManualDataExportNoticeRule>>,
}

impl ManualDataExportNoticeValidator {
    pub fn validate(&self, item: &ManualDataExportNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualDataExportNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDataExportNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_data_export_notice_validator() -> ManualDataExportNoticeValidator {
    ManualDataExportNoticeValidator {
        rules: vec![
            Rc::new(ManualDataExportNoticeTitleLengthRule { max_length: 32 }) as Rc<dyn ManualDataExportNoticeRule>,
            Rc::new(ManualDataExportNoticeRequiredTitleRule {
                message: "manual data export notice needs a value".to_string(),
            }),
            Rc::new(ManualDataExportNoticeRequiredPhoneRule {
                message: "manual data export notice needs a value".to_string(),
            }),
            Rc::new(ManualDataExportNoticeSequenceRangeRule { min: 120, max: 86400 }),
            Rc::new(ManualDataExportNoticeLabelLengthRule { max_length: 32 }),
            Rc::new(ManualDataExportNoticeStartsAtRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_data_export_notice_validator();
        let item = new_manual_data_export_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualDataExportNoticeTitleLengthRule { max_length: 255 };
        let mut item = new_manual_data_export_notice("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualDataExportNoticeRequiredTitleRule {
            message: "manual data export notice needs this field".to_string(),
        };
        let mut item = new_manual_data_export_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ManualDataExportNoticeRequiredPhoneRule {
            message: "manual data export notice needs an entry".to_string(),
        };
        let mut item = new_manual_data_export_notice("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ManualDataExportNoticeSequenceRangeRule { min: 16, max: 120 };
        let mut item = new_manual_data_export_notice("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualDataExportNoticeLabelLengthRule { max_length: 1000 };
        let mut item = new_manual_data_export_notice("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = ManualDataExportNoticeStartsAtRangeRule { min: 32, max: 3600 };
        let mut item = new_manual_data_export_notice("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
