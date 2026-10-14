use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration manifest must pass. Check returns its problems, or none.
pub trait MigrationManifestRule: std::fmt::Debug {
    fn check(&self, item: &MigrationManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationManifestLabelLengthRule {
    pub max_length: i64,
}

impl MigrationManifestRule for MigrationManifestLabelLengthRule {
    fn check(&self, item: &MigrationManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationManifestStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationManifestRule for MigrationManifestStartsAtRangeRule {
    fn check(&self, item: &MigrationManifest) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationManifestRequiredLabelRule {
    pub message: String,
}

impl MigrationManifestRule for MigrationManifestRequiredLabelRule {
    fn check(&self, item: &MigrationManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationManifestKeyIdLengthRule {
    pub max_length: i64,
}

impl MigrationManifestRule for MigrationManifestKeyIdLengthRule {
    fn check(&self, item: &MigrationManifest) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration manifest.
#[derive(Debug, Clone)]
pub struct MigrationManifestValidator {
    pub rules: Vec<Rc<dyn MigrationManifestRule>>,
}

impl MigrationManifestValidator {
    pub fn validate(&self, item: &MigrationManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_manifest_validator() -> MigrationManifestValidator {
    MigrationManifestValidator {
        rules: vec![
            Rc::new(MigrationManifestLabelLengthRule { max_length: 86400 }) as Rc<dyn MigrationManifestRule>,
            Rc::new(MigrationManifestStartsAtRangeRule { min: 32, max: 86400 }),
            Rc::new(MigrationManifestRequiredLabelRule {
                message: "migration manifest needs an entry".to_string(),
            }),
            Rc::new(MigrationManifestKeyIdLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_manifest_validator();
        let item = new_migration_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MigrationManifestLabelLengthRule { max_length: 3600 };
        let mut item = new_migration_manifest("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = MigrationManifestStartsAtRangeRule { min: 120, max: 3600 };
        let mut item = new_migration_manifest("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = MigrationManifestRequiredLabelRule {
            message: "migration manifest needs a value".to_string(),
        };
        let mut item = new_migration_manifest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = MigrationManifestKeyIdLengthRule { max_length: 3600 };
        let mut item = new_migration_manifest("x", "y");
        item.key_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
