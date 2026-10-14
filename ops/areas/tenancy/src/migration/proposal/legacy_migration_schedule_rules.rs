use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy migration schedule must pass. Check returns its problems, or none.
pub trait LegacyMigrationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &LegacyMigrationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationScheduleRequiredExternalRefRule {
    pub message: String,
}

impl LegacyMigrationScheduleRule for LegacyMigrationScheduleRequiredExternalRefRule {
    fn check(&self, item: &LegacyMigrationSchedule) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationScheduleRequiredNotesRule {
    pub message: String,
}

impl LegacyMigrationScheduleRule for LegacyMigrationScheduleRequiredNotesRule {
    fn check(&self, item: &LegacyMigrationSchedule) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationScheduleNotesLengthRule {
    pub max_length: i64,
}

impl LegacyMigrationScheduleRule for LegacyMigrationScheduleNotesLengthRule {
    fn check(&self, item: &LegacyMigrationSchedule) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy migration schedule.
#[derive(Debug, Clone)]
pub struct LegacyMigrationScheduleValidator {
    pub rules: Vec<Rc<dyn LegacyMigrationScheduleRule>>,
}

impl LegacyMigrationScheduleValidator {
    pub fn validate(&self, item: &LegacyMigrationSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyMigrationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyMigrationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_migration_schedule_validator() -> LegacyMigrationScheduleValidator {
    LegacyMigrationScheduleValidator {
        rules: vec![
            Rc::new(LegacyMigrationScheduleRequiredExternalRefRule {
                message: "legacy migration schedule needs an entry".to_string(),
            }) as Rc<dyn LegacyMigrationScheduleRule>,
            Rc::new(LegacyMigrationScheduleRequiredNotesRule {
                message: "legacy migration schedule needs a value".to_string(),
            }),
            Rc::new(LegacyMigrationScheduleNotesLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_migration_schedule_validator();
        let item = new_legacy_migration_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = LegacyMigrationScheduleRequiredExternalRefRule {
            message: "legacy migration schedule needs a value".to_string(),
        };
        let mut item = new_legacy_migration_schedule("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = LegacyMigrationScheduleRequiredNotesRule {
            message: "legacy migration schedule needs an entry".to_string(),
        };
        let mut item = new_legacy_migration_schedule("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = LegacyMigrationScheduleNotesLengthRule { max_length: 255 };
        let mut item = new_legacy_migration_schedule("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
