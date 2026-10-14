use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce snapshot must pass. Check returns its problems, or none.
pub trait NonceSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &NonceSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSnapshotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceSnapshotRule for NonceSnapshotUpdatedAtRangeRule {
    fn check(&self, item: &NonceSnapshot) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSnapshotCodePrefixRule {
    pub prefix: String,
}

impl NonceSnapshotRule for NonceSnapshotCodePrefixRule {
    fn check(&self, item: &NonceSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSnapshotOwnerIdLengthRule {
    pub max_length: i64,
}

impl NonceSnapshotRule for NonceSnapshotOwnerIdLengthRule {
    fn check(&self, item: &NonceSnapshot) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSnapshotNameLengthRule {
    pub max_length: i64,
}

impl NonceSnapshotRule for NonceSnapshotNameLengthRule {
    fn check(&self, item: &NonceSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce snapshot.
#[derive(Debug, Clone)]
pub struct NonceSnapshotValidator {
    pub rules: Vec<Rc<dyn NonceSnapshotRule>>,
}

impl NonceSnapshotValidator {
    pub fn validate(&self, item: &NonceSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_snapshot_validator() -> NonceSnapshotValidator {
    NonceSnapshotValidator {
        rules: vec![
            Rc::new(NonceSnapshotUpdatedAtRangeRule { min: 32, max: 64 }) as Rc<dyn NonceSnapshotRule>,
            Rc::new(NonceSnapshotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(NonceSnapshotOwnerIdLengthRule { max_length: 120 }),
            Rc::new(NonceSnapshotNameLengthRule { max_length: 3600 }),
        ],
    }
}
