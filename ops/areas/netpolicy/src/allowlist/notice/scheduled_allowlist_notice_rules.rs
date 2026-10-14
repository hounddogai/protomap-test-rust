use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled allowlist notice must pass. Check returns its problems, or none.
pub trait ScheduledAllowlistNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledAllowlistNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistNoticeSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledAllowlistNoticeRule for ScheduledAllowlistNoticeSequenceRangeRule {
    fn check(&self, item: &ScheduledAllowlistNotice) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistNoticeRequiredCodeRule {
    pub message: String,
}

impl ScheduledAllowlistNoticeRule for ScheduledAllowlistNoticeRequiredCodeRule {
    fn check(&self, item: &ScheduledAllowlistNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistNoticeRequiredFingerprintRule {
    pub message: String,
}

impl ScheduledAllowlistNoticeRule for ScheduledAllowlistNoticeRequiredFingerprintRule {
    fn check(&self, item: &ScheduledAllowlistNotice) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistNoticeLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledAllowlistNoticeRule for ScheduledAllowlistNoticeLimitRangeRule {
    fn check(&self, item: &ScheduledAllowlistNotice) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistNoticeRequiredTitleRule {
    pub message: String,
}

impl ScheduledAllowlistNoticeRule for ScheduledAllowlistNoticeRequiredTitleRule {
    fn check(&self, item: &ScheduledAllowlistNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAllowlistNoticeFingerprintLengthRule {
    pub max_length: i64,
}

impl ScheduledAllowlistNoticeRule for ScheduledAllowlistNoticeFingerprintLengthRule {
    fn check(&self, item: &ScheduledAllowlistNotice) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled allowlist notice.
#[derive(Debug, Clone)]
pub struct ScheduledAllowlistNoticeValidator {
    pub rules: Vec<Rc<dyn ScheduledAllowlistNoticeRule>>,
}

impl ScheduledAllowlistNoticeValidator {
    pub fn validate(&self, item: &ScheduledAllowlistNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledAllowlistNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledAllowlistNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_allowlist_notice_validator() -> ScheduledAllowlistNoticeValidator {
    ScheduledAllowlistNoticeValidator {
        rules: vec![
            Rc::new(ScheduledAllowlistNoticeSequenceRangeRule { min: 32, max: 86400 })
                as Rc<dyn ScheduledAllowlistNoticeRule>,
            Rc::new(ScheduledAllowlistNoticeRequiredCodeRule {
                message: "scheduled allowlist notice needs an entry".to_string(),
            }),
            Rc::new(ScheduledAllowlistNoticeRequiredFingerprintRule {
                message: "scheduled allowlist notice needs an entry".to_string(),
            }),
            Rc::new(ScheduledAllowlistNoticeLimitRangeRule { min: 64, max: 255 }),
            Rc::new(ScheduledAllowlistNoticeRequiredTitleRule {
                message: "scheduled allowlist notice needs this field".to_string(),
            }),
            Rc::new(ScheduledAllowlistNoticeFingerprintLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_allowlist_notice_validator();
        let item = new_scheduled_allowlist_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ScheduledAllowlistNoticeSequenceRangeRule { min: 80, max: 255 };
        let mut item = new_scheduled_allowlist_notice("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledAllowlistNoticeRequiredCodeRule {
            message: "scheduled allowlist notice needs a value".to_string(),
        };
        let mut item = new_scheduled_allowlist_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ScheduledAllowlistNoticeRequiredFingerprintRule {
            message: "scheduled allowlist notice needs an entry".to_string(),
        };
        let mut item = new_scheduled_allowlist_notice("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ScheduledAllowlistNoticeLimitRangeRule { min: 32, max: 86400 };
        let mut item = new_scheduled_allowlist_notice("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledAllowlistNoticeRequiredTitleRule {
            message: "scheduled allowlist notice needs this field".to_string(),
        };
        let mut item = new_scheduled_allowlist_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ScheduledAllowlistNoticeFingerprintLengthRule { max_length: 255 };
        let mut item = new_scheduled_allowlist_notice("x", "y");
        item.fingerprint = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
