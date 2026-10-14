use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration draft must pass. Check returns its problems, or none.
pub trait MigrationDraftRule: std::fmt::Debug {
    fn check(&self, item: &MigrationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDraftRequiredCodeRule {
    pub message: String,
}

impl MigrationDraftRule for MigrationDraftRequiredCodeRule {
    fn check(&self, item: &MigrationDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDraftCodeLengthRule {
    pub max_length: i64,
}

impl MigrationDraftRule for MigrationDraftCodeLengthRule {
    fn check(&self, item: &MigrationDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDraftRequiredPhoneRule {
    pub message: String,
}

impl MigrationDraftRule for MigrationDraftRequiredPhoneRule {
    fn check(&self, item: &MigrationDraft) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDraftCodePrefixRule {
    pub prefix: String,
}

impl MigrationDraftRule for MigrationDraftCodePrefixRule {
    fn check(&self, item: &MigrationDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDraftQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationDraftRule for MigrationDraftQuantityRangeRule {
    fn check(&self, item: &MigrationDraft) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationDraftDescriptionLengthRule {
    pub max_length: i64,
}

impl MigrationDraftRule for MigrationDraftDescriptionLengthRule {
    fn check(&self, item: &MigrationDraft) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration draft.
#[derive(Debug, Clone)]
pub struct MigrationDraftValidator {
    pub rules: Vec<Rc<dyn MigrationDraftRule>>,
}

impl MigrationDraftValidator {
    pub fn validate(&self, item: &MigrationDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_draft_validator() -> MigrationDraftValidator {
    MigrationDraftValidator {
        rules: vec![
            Rc::new(MigrationDraftRequiredCodeRule {
                message: "migration draft needs this field".to_string(),
            }) as Rc<dyn MigrationDraftRule>,
            Rc::new(MigrationDraftCodeLengthRule { max_length: 120 }),
            Rc::new(MigrationDraftRequiredPhoneRule {
                message: "migration draft needs a value".to_string(),
            }),
            Rc::new(MigrationDraftCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(MigrationDraftQuantityRangeRule { min: 16, max: 64 }),
            Rc::new(MigrationDraftDescriptionLengthRule { max_length: 120 }),
        ],
    }
}
