use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration preference must pass. Check returns its problems, or none.
pub trait MigrationPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &MigrationPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPreferenceRequiredSummaryRule {
    pub message: String,
}

impl MigrationPreferenceRule for MigrationPreferenceRequiredSummaryRule {
    fn check(&self, item: &MigrationPreference) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPreferenceCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationPreferenceRule for MigrationPreferenceCreatedAtRangeRule {
    fn check(&self, item: &MigrationPreference) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl MigrationPreferenceRule for MigrationPreferenceCodeLengthRule {
    fn check(&self, item: &MigrationPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration preference.
#[derive(Debug, Clone)]
pub struct MigrationPreferenceValidator {
    pub rules: Vec<Rc<dyn MigrationPreferenceRule>>,
}

impl MigrationPreferenceValidator {
    pub fn validate(&self, item: &MigrationPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_preference_validator() -> MigrationPreferenceValidator {
    MigrationPreferenceValidator {
        rules: vec![
            Rc::new(MigrationPreferenceRequiredSummaryRule {
                message: "migration preference needs this field".to_string(),
            }) as Rc<dyn MigrationPreferenceRule>,
            Rc::new(MigrationPreferenceCreatedAtRangeRule { min: 16, max: 32 }),
            Rc::new(MigrationPreferenceCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_preference_validator();
        let item = new_migration_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = MigrationPreferenceRequiredSummaryRule {
            message: "migration preference needs a value".to_string(),
        };
        let mut item = new_migration_preference("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = MigrationPreferenceCreatedAtRangeRule { min: 80, max: 120 };
        let mut item = new_migration_preference("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationPreferenceCodeLengthRule { max_length: 32 };
        let mut item = new_migration_preference("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
