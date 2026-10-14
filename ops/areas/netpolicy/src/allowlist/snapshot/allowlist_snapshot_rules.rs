use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist snapshot must pass. Check returns its problems, or none.
pub trait AllowlistSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl AllowlistSnapshotRule for AllowlistSnapshotTitleLengthRule {
    fn check(&self, item: &AllowlistSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSnapshotRequiredTitleRule {
    pub message: String,
}

impl AllowlistSnapshotRule for AllowlistSnapshotRequiredTitleRule {
    fn check(&self, item: &AllowlistSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl AllowlistSnapshotRule for AllowlistSnapshotLabelLengthRule {
    fn check(&self, item: &AllowlistSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist snapshot.
#[derive(Debug, Clone)]
pub struct AllowlistSnapshotValidator {
    pub rules: Vec<Rc<dyn AllowlistSnapshotRule>>,
}

impl AllowlistSnapshotValidator {
    pub fn validate(&self, item: &AllowlistSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_snapshot_validator() -> AllowlistSnapshotValidator {
    AllowlistSnapshotValidator {
        rules: vec![
            Rc::new(AllowlistSnapshotTitleLengthRule { max_length: 64 }) as Rc<dyn AllowlistSnapshotRule>,
            Rc::new(AllowlistSnapshotRequiredTitleRule {
                message: "allowlist snapshot needs a value".to_string(),
            }),
            Rc::new(AllowlistSnapshotLabelLengthRule { max_length: 3600 }),
        ],
    }
}
