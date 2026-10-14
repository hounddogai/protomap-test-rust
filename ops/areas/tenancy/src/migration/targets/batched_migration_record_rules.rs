use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched migration record must pass. Check returns its problems, or none.
pub trait BatchedMigrationRecordRule: std::fmt::Debug {
    fn check(&self, item: &BatchedMigrationRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRecordRequiredLabelRule {
    pub message: String,
}

impl BatchedMigrationRecordRule for BatchedMigrationRecordRequiredLabelRule {
    fn check(&self, item: &BatchedMigrationRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRecordLabelLengthRule {
    pub max_length: i64,
}

impl BatchedMigrationRecordRule for BatchedMigrationRecordLabelLengthRule {
    fn check(&self, item: &BatchedMigrationRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRecordCodeLengthRule {
    pub max_length: i64,
}

impl BatchedMigrationRecordRule for BatchedMigrationRecordCodeLengthRule {
    fn check(&self, item: &BatchedMigrationRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRecordCodePrefixRule {
    pub prefix: String,
}

impl BatchedMigrationRecordRule for BatchedMigrationRecordCodePrefixRule {
    fn check(&self, item: &BatchedMigrationRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched migration record.
#[derive(Debug, Clone)]
pub struct BatchedMigrationRecordValidator {
    pub rules: Vec<Rc<dyn BatchedMigrationRecordRule>>,
}

impl BatchedMigrationRecordValidator {
    pub fn validate(&self, item: &BatchedMigrationRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedMigrationRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedMigrationRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_migration_record_validator() -> BatchedMigrationRecordValidator {
    BatchedMigrationRecordValidator {
        rules: vec![
            Rc::new(BatchedMigrationRecordRequiredLabelRule {
                message: "batched migration record needs a value".to_string(),
            }) as Rc<dyn BatchedMigrationRecordRule>,
            Rc::new(BatchedMigrationRecordLabelLengthRule { max_length: 64 }),
            Rc::new(BatchedMigrationRecordCodeLengthRule { max_length: 255 }),
            Rc::new(BatchedMigrationRecordCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
