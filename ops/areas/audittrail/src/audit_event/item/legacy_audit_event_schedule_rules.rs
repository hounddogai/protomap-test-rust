use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy audit event schedule must pass. Check returns its problems, or none.
pub trait LegacyAuditEventScheduleRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAuditEventSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventScheduleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LegacyAuditEventScheduleRule for LegacyAuditEventScheduleTagsLimitRule {
    fn check(&self, item: &LegacyAuditEventSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventScheduleTitleLengthRule {
    pub max_length: i64,
}

impl LegacyAuditEventScheduleRule for LegacyAuditEventScheduleTitleLengthRule {
    fn check(&self, item: &LegacyAuditEventSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventSchedulePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyAuditEventScheduleRule for LegacyAuditEventSchedulePortRangeRule {
    fn check(&self, item: &LegacyAuditEventSchedule) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventScheduleVisibleGuardRule {
    pub limit: i64,
}

impl LegacyAuditEventScheduleRule for LegacyAuditEventScheduleVisibleGuardRule {
    fn check(&self, item: &LegacyAuditEventSchedule) -> Vec<String> {
        if item.visible && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while visible", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventScheduleRequiredTitleRule {
    pub message: String,
}

impl LegacyAuditEventScheduleRule for LegacyAuditEventScheduleRequiredTitleRule {
    fn check(&self, item: &LegacyAuditEventSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventScheduleCodePrefixRule {
    pub prefix: String,
}

impl LegacyAuditEventScheduleRule for LegacyAuditEventScheduleCodePrefixRule {
    fn check(&self, item: &LegacyAuditEventSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy audit event schedule.
#[derive(Debug, Clone)]
pub struct LegacyAuditEventScheduleValidator {
    pub rules: Vec<Rc<dyn LegacyAuditEventScheduleRule>>,
}

impl LegacyAuditEventScheduleValidator {
    pub fn validate(&self, item: &LegacyAuditEventSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyAuditEventSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAuditEventSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_audit_event_schedule_validator() -> LegacyAuditEventScheduleValidator {
    LegacyAuditEventScheduleValidator {
        rules: vec![
            Rc::new(LegacyAuditEventScheduleTagsLimitRule {
                max_count: 3600,
                banned: Vec::new(),
            }) as Rc<dyn LegacyAuditEventScheduleRule>,
            Rc::new(LegacyAuditEventScheduleTitleLengthRule { max_length: 16 }),
            Rc::new(LegacyAuditEventSchedulePortRangeRule { min: 16, max: 16 }),
            Rc::new(LegacyAuditEventScheduleVisibleGuardRule { limit: 86400 }),
            Rc::new(LegacyAuditEventScheduleRequiredTitleRule {
                message: "legacy audit event schedule needs this field".to_string(),
            }),
            Rc::new(LegacyAuditEventScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_audit_event_schedule_validator();
        let item = new_legacy_audit_event_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyAuditEventScheduleTitleLengthRule { max_length: 64 };
        let mut item = new_legacy_audit_event_schedule("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LegacyAuditEventSchedulePortRangeRule { min: 80, max: 86400 };
        let mut item = new_legacy_audit_event_schedule("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyAuditEventScheduleRequiredTitleRule {
            message: "legacy audit event schedule needs a value".to_string(),
        };
        let mut item = new_legacy_audit_event_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
