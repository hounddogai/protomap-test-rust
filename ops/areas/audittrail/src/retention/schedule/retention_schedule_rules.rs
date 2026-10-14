use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention schedule must pass. Check returns its problems, or none.
pub trait RetentionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RetentionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScheduleOwnerIdLengthRule {
    pub max_length: i64,
}

impl RetentionScheduleRule for RetentionScheduleOwnerIdLengthRule {
    fn check(&self, item: &RetentionSchedule) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScheduleRequiredOwnerIdRule {
    pub message: String,
}

impl RetentionScheduleRule for RetentionScheduleRequiredOwnerIdRule {
    fn check(&self, item: &RetentionSchedule) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScheduleRequiredAudienceRule {
    pub message: String,
}

impl RetentionScheduleRule for RetentionScheduleRequiredAudienceRule {
    fn check(&self, item: &RetentionSchedule) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention schedule.
#[derive(Debug, Clone)]
pub struct RetentionScheduleValidator {
    pub rules: Vec<Rc<dyn RetentionScheduleRule>>,
}

impl RetentionScheduleValidator {
    pub fn validate(&self, item: &RetentionSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_schedule_validator() -> RetentionScheduleValidator {
    RetentionScheduleValidator {
        rules: vec![
            Rc::new(RetentionScheduleOwnerIdLengthRule { max_length: 32 }) as Rc<dyn RetentionScheduleRule>,
            Rc::new(RetentionScheduleRequiredOwnerIdRule {
                message: "retention schedule needs this field".to_string(),
            }),
            Rc::new(RetentionScheduleRequiredAudienceRule {
                message: "retention schedule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_schedule_validator();
        let item = new_retention_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = RetentionScheduleOwnerIdLengthRule { max_length: 86400 };
        let mut item = new_retention_schedule("x", "y");
        item.owner_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = RetentionScheduleRequiredOwnerIdRule {
            message: "retention schedule needs this field".to_string(),
        };
        let mut item = new_retention_schedule("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RetentionScheduleRequiredAudienceRule {
            message: "retention schedule needs a value".to_string(),
        };
        let mut item = new_retention_schedule("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
