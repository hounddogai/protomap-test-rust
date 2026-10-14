use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy migration checkpoint must pass. Check returns its problems, or none.
pub trait LegacyMigrationCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &LegacyMigrationCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationCheckpointWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyMigrationCheckpointRule for LegacyMigrationCheckpointWindowDaysRangeRule {
    fn check(&self, item: &LegacyMigrationCheckpoint) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl LegacyMigrationCheckpointRule for LegacyMigrationCheckpointLabelLengthRule {
    fn check(&self, item: &LegacyMigrationCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationCheckpointAudienceLengthRule {
    pub max_length: i64,
}

impl LegacyMigrationCheckpointRule for LegacyMigrationCheckpointAudienceLengthRule {
    fn check(&self, item: &LegacyMigrationCheckpoint) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationCheckpointRequiredCodeRule {
    pub message: String,
}

impl LegacyMigrationCheckpointRule for LegacyMigrationCheckpointRequiredCodeRule {
    fn check(&self, item: &LegacyMigrationCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy migration checkpoint.
#[derive(Debug, Clone)]
pub struct LegacyMigrationCheckpointValidator {
    pub rules: Vec<Rc<dyn LegacyMigrationCheckpointRule>>,
}

impl LegacyMigrationCheckpointValidator {
    pub fn validate(&self, item: &LegacyMigrationCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyMigrationCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyMigrationCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_migration_checkpoint_validator() -> LegacyMigrationCheckpointValidator {
    LegacyMigrationCheckpointValidator {
        rules: vec![
            Rc::new(LegacyMigrationCheckpointWindowDaysRangeRule { min: 16, max: 64 })
                as Rc<dyn LegacyMigrationCheckpointRule>,
            Rc::new(LegacyMigrationCheckpointLabelLengthRule { max_length: 32 }),
            Rc::new(LegacyMigrationCheckpointAudienceLengthRule { max_length: 3600 }),
            Rc::new(LegacyMigrationCheckpointRequiredCodeRule {
                message: "legacy migration checkpoint needs this field".to_string(),
            }),
        ],
    }
}
