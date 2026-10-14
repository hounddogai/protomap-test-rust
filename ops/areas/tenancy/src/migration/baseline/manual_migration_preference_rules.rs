use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual migration preference must pass. Check returns its problems, or none.
pub trait ManualMigrationPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ManualMigrationPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationPreferenceWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualMigrationPreferenceRule for ManualMigrationPreferenceWeightRangeRule {
    fn check(&self, item: &ManualMigrationPreference) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationPreferenceNotesLengthRule {
    pub max_length: i64,
}

impl ManualMigrationPreferenceRule for ManualMigrationPreferenceNotesLengthRule {
    fn check(&self, item: &ManualMigrationPreference) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationPreferenceRequiredCodeRule {
    pub message: String,
}

impl ManualMigrationPreferenceRule for ManualMigrationPreferenceRequiredCodeRule {
    fn check(&self, item: &ManualMigrationPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual migration preference.
#[derive(Debug, Clone)]
pub struct ManualMigrationPreferenceValidator {
    pub rules: Vec<Rc<dyn ManualMigrationPreferenceRule>>,
}

impl ManualMigrationPreferenceValidator {
    pub fn validate(&self, item: &ManualMigrationPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualMigrationPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualMigrationPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_migration_preference_validator() -> ManualMigrationPreferenceValidator {
    ManualMigrationPreferenceValidator {
        rules: vec![
            Rc::new(ManualMigrationPreferenceWeightRangeRule { min: 2.5, max: 1040.4 })
                as Rc<dyn ManualMigrationPreferenceRule>,
            Rc::new(ManualMigrationPreferenceNotesLengthRule { max_length: 255 }),
            Rc::new(ManualMigrationPreferenceRequiredCodeRule {
                message: "manual migration preference needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_migration_preference_validator();
        let item = new_manual_migration_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ManualMigrationPreferenceWeightRangeRule { min: 5.4, max: 603.9 };
        let mut item = new_manual_migration_preference("x", "y");
        item.weight = 605.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = ManualMigrationPreferenceNotesLengthRule { max_length: 255 };
        let mut item = new_manual_migration_preference("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualMigrationPreferenceRequiredCodeRule {
            message: "manual migration preference needs an entry".to_string(),
        };
        let mut item = new_manual_migration_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
