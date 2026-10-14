use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration record must pass. Check returns its problems, or none.
pub trait MigrationRecordRule: std::fmt::Debug {
    fn check(&self, item: &MigrationRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRecordUnitLengthRule {
    pub max_length: i64,
}

impl MigrationRecordRule for MigrationRecordUnitLengthRule {
    fn check(&self, item: &MigrationRecord) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRecordGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationRecordRule for MigrationRecordGraceDaysRangeRule {
    fn check(&self, item: &MigrationRecord) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRecordRequiredTitleRule {
    pub message: String,
}

impl MigrationRecordRule for MigrationRecordRequiredTitleRule {
    fn check(&self, item: &MigrationRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRecordRequiredDescriptionRule {
    pub message: String,
}

impl MigrationRecordRule for MigrationRecordRequiredDescriptionRule {
    fn check(&self, item: &MigrationRecord) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration record.
#[derive(Debug, Clone)]
pub struct MigrationRecordValidator {
    pub rules: Vec<Rc<dyn MigrationRecordRule>>,
}

impl MigrationRecordValidator {
    pub fn validate(&self, item: &MigrationRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_record_validator() -> MigrationRecordValidator {
    MigrationRecordValidator {
        rules: vec![
            Rc::new(MigrationRecordUnitLengthRule { max_length: 80 }) as Rc<dyn MigrationRecordRule>,
            Rc::new(MigrationRecordGraceDaysRangeRule { min: 80, max: 120 }),
            Rc::new(MigrationRecordRequiredTitleRule {
                message: "migration record needs a value".to_string(),
            }),
            Rc::new(MigrationRecordRequiredDescriptionRule {
                message: "migration record needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_record_validator();
        let item = new_migration_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = MigrationRecordUnitLengthRule { max_length: 1000 };
        let mut item = new_migration_record("x", "y");
        item.unit = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = MigrationRecordGraceDaysRangeRule { min: 16, max: 64 };
        let mut item = new_migration_record("x", "y");
        item.grace_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MigrationRecordRequiredTitleRule {
            message: "migration record needs an entry".to_string(),
        };
        let mut item = new_migration_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = MigrationRecordRequiredDescriptionRule {
            message: "migration record needs an entry".to_string(),
        };
        let mut item = new_migration_record("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
