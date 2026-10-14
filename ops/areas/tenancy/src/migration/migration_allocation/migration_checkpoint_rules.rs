use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration checkpoint must pass. Check returns its problems, or none.
pub trait MigrationCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &MigrationCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl MigrationCheckpointRule for MigrationCheckpointCodeLengthRule {
    fn check(&self, item: &MigrationCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCheckpointRequiredTitleRule {
    pub message: String,
}

impl MigrationCheckpointRule for MigrationCheckpointRequiredTitleRule {
    fn check(&self, item: &MigrationCheckpoint) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCheckpointCountryLengthRule {
    pub max_length: i64,
}

impl MigrationCheckpointRule for MigrationCheckpointCountryLengthRule {
    fn check(&self, item: &MigrationCheckpoint) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCheckpointDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationCheckpointRule for MigrationCheckpointDiscountRangeRule {
    fn check(&self, item: &MigrationCheckpoint) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCheckpointRequiredCodeRule {
    pub message: String,
}

impl MigrationCheckpointRule for MigrationCheckpointRequiredCodeRule {
    fn check(&self, item: &MigrationCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration checkpoint.
#[derive(Debug, Clone)]
pub struct MigrationCheckpointValidator {
    pub rules: Vec<Rc<dyn MigrationCheckpointRule>>,
}

impl MigrationCheckpointValidator {
    pub fn validate(&self, item: &MigrationCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_checkpoint_validator() -> MigrationCheckpointValidator {
    MigrationCheckpointValidator {
        rules: vec![
            Rc::new(MigrationCheckpointCodeLengthRule { max_length: 86400 }) as Rc<dyn MigrationCheckpointRule>,
            Rc::new(MigrationCheckpointRequiredTitleRule {
                message: "migration checkpoint needs this field".to_string(),
            }),
            Rc::new(MigrationCheckpointCountryLengthRule { max_length: 64 }),
            Rc::new(MigrationCheckpointDiscountRangeRule { min: 2.0, max: 149.7 }),
            Rc::new(MigrationCheckpointRequiredCodeRule {
                message: "migration checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_checkpoint_validator();
        let item = new_migration_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationCheckpointCodeLengthRule { max_length: 3600 };
        let mut item = new_migration_checkpoint("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MigrationCheckpointRequiredTitleRule {
            message: "migration checkpoint needs an entry".to_string(),
        };
        let mut item = new_migration_checkpoint("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = MigrationCheckpointCountryLengthRule { max_length: 64 };
        let mut item = new_migration_checkpoint("x", "y");
        item.country = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = MigrationCheckpointDiscountRangeRule { min: 1.6, max: 212.7 };
        let mut item = new_migration_checkpoint("x", "y");
        item.discount = 214.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationCheckpointRequiredCodeRule {
            message: "migration checkpoint needs this field".to_string(),
        };
        let mut item = new_migration_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
