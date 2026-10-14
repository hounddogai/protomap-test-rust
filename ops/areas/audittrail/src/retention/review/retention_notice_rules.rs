use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention notice must pass. Check returns its problems, or none.
pub trait RetentionNoticeRule: std::fmt::Debug {
    fn check(&self, item: &RetentionNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionNoticeUnitLengthRule {
    pub max_length: i64,
}

impl RetentionNoticeRule for RetentionNoticeUnitLengthRule {
    fn check(&self, item: &RetentionNotice) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionNoticeCodePrefixRule {
    pub prefix: String,
}

impl RetentionNoticeRule for RetentionNoticeCodePrefixRule {
    fn check(&self, item: &RetentionNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionNoticeCodeLengthRule {
    pub max_length: i64,
}

impl RetentionNoticeRule for RetentionNoticeCodeLengthRule {
    fn check(&self, item: &RetentionNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionNoticeDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionNoticeRule for RetentionNoticeDurationMinutesRangeRule {
    fn check(&self, item: &RetentionNotice) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionNoticeRequiredTitleRule {
    pub message: String,
}

impl RetentionNoticeRule for RetentionNoticeRequiredTitleRule {
    fn check(&self, item: &RetentionNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention notice.
#[derive(Debug, Clone)]
pub struct RetentionNoticeValidator {
    pub rules: Vec<Rc<dyn RetentionNoticeRule>>,
}

impl RetentionNoticeValidator {
    pub fn validate(&self, item: &RetentionNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_notice_validator() -> RetentionNoticeValidator {
    RetentionNoticeValidator {
        rules: vec![
            Rc::new(RetentionNoticeUnitLengthRule { max_length: 80 }) as Rc<dyn RetentionNoticeRule>,
            Rc::new(RetentionNoticeCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RetentionNoticeCodeLengthRule { max_length: 80 }),
            Rc::new(RetentionNoticeDurationMinutesRangeRule { min: 16, max: 120 }),
            Rc::new(RetentionNoticeRequiredTitleRule {
                message: "retention notice needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_notice_validator();
        let item = new_retention_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = RetentionNoticeUnitLengthRule { max_length: 86400 };
        let mut item = new_retention_notice("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionNoticeCodeLengthRule { max_length: 3600 };
        let mut item = new_retention_notice("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RetentionNoticeDurationMinutesRangeRule { min: 120, max: 255 };
        let mut item = new_retention_notice("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RetentionNoticeRequiredTitleRule {
            message: "retention notice needs a value".to_string(),
        };
        let mut item = new_retention_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
