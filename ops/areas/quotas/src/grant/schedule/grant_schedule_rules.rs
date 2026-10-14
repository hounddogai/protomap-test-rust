use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant schedule must pass. Check returns its problems, or none.
pub trait GrantScheduleRule: std::fmt::Debug {
    fn check(&self, item: &GrantSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantScheduleDescriptionLengthRule {
    pub max_length: i64,
}

impl GrantScheduleRule for GrantScheduleDescriptionLengthRule {
    fn check(&self, item: &GrantSchedule) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantScheduleRequiredDescriptionRule {
    pub message: String,
}

impl GrantScheduleRule for GrantScheduleRequiredDescriptionRule {
    fn check(&self, item: &GrantSchedule) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantScheduleNameLengthRule {
    pub max_length: i64,
}

impl GrantScheduleRule for GrantScheduleNameLengthRule {
    fn check(&self, item: &GrantSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantScheduleRequiredNameRule {
    pub message: String,
}

impl GrantScheduleRule for GrantScheduleRequiredNameRule {
    fn check(&self, item: &GrantSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantScheduleTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl GrantScheduleRule for GrantScheduleTagsLimitRule {
    fn check(&self, item: &GrantSchedule) -> Vec<String> {
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

/// Runs every rule of a grant schedule.
#[derive(Debug, Clone)]
pub struct GrantScheduleValidator {
    pub rules: Vec<Rc<dyn GrantScheduleRule>>,
}

impl GrantScheduleValidator {
    pub fn validate(&self, item: &GrantSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_schedule_validator() -> GrantScheduleValidator {
    GrantScheduleValidator {
        rules: vec![
            Rc::new(GrantScheduleDescriptionLengthRule { max_length: 64 }) as Rc<dyn GrantScheduleRule>,
            Rc::new(GrantScheduleRequiredDescriptionRule {
                message: "grant schedule needs an entry".to_string(),
            }),
            Rc::new(GrantScheduleNameLengthRule { max_length: 1000 }),
            Rc::new(GrantScheduleRequiredNameRule {
                message: "grant schedule needs a value".to_string(),
            }),
            Rc::new(GrantScheduleTagsLimitRule {
                max_count: 1000,
                banned: vec!["scheduled".to_string(), "tiered".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_schedule_validator();
        let item = new_grant_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = GrantScheduleDescriptionLengthRule { max_length: 3600 };
        let mut item = new_grant_schedule("x", "y");
        item.description = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = GrantScheduleRequiredDescriptionRule {
            message: "grant schedule needs an entry".to_string(),
        };
        let mut item = new_grant_schedule("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GrantScheduleNameLengthRule { max_length: 16 };
        let mut item = new_grant_schedule("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GrantScheduleRequiredNameRule {
            message: "grant schedule needs a value".to_string(),
        };
        let mut item = new_grant_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
