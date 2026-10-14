use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default rollback snapshot must pass. Check returns its problems, or none.
pub trait DefaultRollbackSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRollbackSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackSnapshotCodePrefixRule {
    pub prefix: String,
}

impl DefaultRollbackSnapshotRule for DefaultRollbackSnapshotCodePrefixRule {
    fn check(&self, item: &DefaultRollbackSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultRollbackSnapshotRule for DefaultRollbackSnapshotAllowedCodeRule {
    fn check(&self, item: &DefaultRollbackSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl DefaultRollbackSnapshotRule for DefaultRollbackSnapshotLabelLengthRule {
    fn check(&self, item: &DefaultRollbackSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rollback snapshot.
#[derive(Debug, Clone)]
pub struct DefaultRollbackSnapshotValidator {
    pub rules: Vec<Rc<dyn DefaultRollbackSnapshotRule>>,
}

impl DefaultRollbackSnapshotValidator {
    pub fn validate(&self, item: &DefaultRollbackSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRollbackSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRollbackSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rollback_snapshot_validator() -> DefaultRollbackSnapshotValidator {
    DefaultRollbackSnapshotValidator {
        rules: vec![
            Rc::new(DefaultRollbackSnapshotCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn DefaultRollbackSnapshotRule>,
            Rc::new(DefaultRollbackSnapshotAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "scheduled".to_string()],
            }),
            Rc::new(DefaultRollbackSnapshotLabelLengthRule { max_length: 86400 }),
        ],
    }
}
