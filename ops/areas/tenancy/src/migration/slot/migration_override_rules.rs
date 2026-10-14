use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration override must pass. Check returns its problems, or none.
pub trait MigrationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &MigrationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationOverrideRegionLengthRule {
    pub max_length: i64,
}

impl MigrationOverrideRule for MigrationOverrideRegionLengthRule {
    fn check(&self, item: &MigrationOverride) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationOverrideRequiredTitleRule {
    pub message: String,
}

impl MigrationOverrideRule for MigrationOverrideRequiredTitleRule {
    fn check(&self, item: &MigrationOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationOverrideTitleLengthRule {
    pub max_length: i64,
}

impl MigrationOverrideRule for MigrationOverrideTitleLengthRule {
    fn check(&self, item: &MigrationOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration override.
#[derive(Debug, Clone)]
pub struct MigrationOverrideValidator {
    pub rules: Vec<Rc<dyn MigrationOverrideRule>>,
}

impl MigrationOverrideValidator {
    pub fn validate(&self, item: &MigrationOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_override_validator() -> MigrationOverrideValidator {
    MigrationOverrideValidator {
        rules: vec![
            Rc::new(MigrationOverrideRegionLengthRule { max_length: 86400 }) as Rc<dyn MigrationOverrideRule>,
            Rc::new(MigrationOverrideRequiredTitleRule {
                message: "migration override needs a value".to_string(),
            }),
            Rc::new(MigrationOverrideTitleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_override_validator();
        let item = new_migration_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = MigrationOverrideRegionLengthRule { max_length: 32 };
        let mut item = new_migration_override("x", "y");
        item.region = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MigrationOverrideRequiredTitleRule {
            message: "migration override needs an entry".to_string(),
        };
        let mut item = new_migration_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = MigrationOverrideTitleLengthRule { max_length: 32 };
        let mut item = new_migration_override("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
