use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant snapshot must pass. Check returns its problems, or none.
pub trait GrantSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &GrantSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl GrantSnapshotRule for GrantSnapshotTitleLengthRule {
    fn check(&self, item: &GrantSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSnapshotUnitLengthRule {
    pub max_length: i64,
}

impl GrantSnapshotRule for GrantSnapshotUnitLengthRule {
    fn check(&self, item: &GrantSnapshot) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSnapshotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantSnapshotRule for GrantSnapshotRevisionRangeRule {
    fn check(&self, item: &GrantSnapshot) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSnapshotRequiredUnitRule {
    pub message: String,
}

impl GrantSnapshotRule for GrantSnapshotRequiredUnitRule {
    fn check(&self, item: &GrantSnapshot) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSnapshotCodePrefixRule {
    pub prefix: String,
}

impl GrantSnapshotRule for GrantSnapshotCodePrefixRule {
    fn check(&self, item: &GrantSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant snapshot.
#[derive(Debug, Clone)]
pub struct GrantSnapshotValidator {
    pub rules: Vec<Rc<dyn GrantSnapshotRule>>,
}

impl GrantSnapshotValidator {
    pub fn validate(&self, item: &GrantSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_snapshot_validator() -> GrantSnapshotValidator {
    GrantSnapshotValidator {
        rules: vec![
            Rc::new(GrantSnapshotTitleLengthRule { max_length: 64 }) as Rc<dyn GrantSnapshotRule>,
            Rc::new(GrantSnapshotUnitLengthRule { max_length: 32 }),
            Rc::new(GrantSnapshotRevisionRangeRule { min: 32, max: 120 }),
            Rc::new(GrantSnapshotRequiredUnitRule {
                message: "grant snapshot needs this field".to_string(),
            }),
            Rc::new(GrantSnapshotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
