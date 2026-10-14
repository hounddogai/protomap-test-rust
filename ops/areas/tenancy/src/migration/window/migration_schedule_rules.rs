use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration schedule must pass. Check returns its problems, or none.
pub trait MigrationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &MigrationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScheduleRequiredReviewerRule {
    pub message: String,
}

impl MigrationScheduleRule for MigrationScheduleRequiredReviewerRule {
    fn check(&self, item: &MigrationSchedule) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScheduleRequiredTitleRule {
    pub message: String,
}

impl MigrationScheduleRule for MigrationScheduleRequiredTitleRule {
    fn check(&self, item: &MigrationSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScheduleCodePrefixRule {
    pub prefix: String,
}

impl MigrationScheduleRule for MigrationScheduleCodePrefixRule {
    fn check(&self, item: &MigrationSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration schedule.
#[derive(Debug, Clone)]
pub struct MigrationScheduleValidator {
    pub rules: Vec<Rc<dyn MigrationScheduleRule>>,
}

impl MigrationScheduleValidator {
    pub fn validate(&self, item: &MigrationSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_schedule_validator() -> MigrationScheduleValidator {
    MigrationScheduleValidator {
        rules: vec![
            Rc::new(MigrationScheduleRequiredReviewerRule {
                message: "migration schedule needs a value".to_string(),
            }) as Rc<dyn MigrationScheduleRule>,
            Rc::new(MigrationScheduleRequiredTitleRule {
                message: "migration schedule needs this field".to_string(),
            }),
            Rc::new(MigrationScheduleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
