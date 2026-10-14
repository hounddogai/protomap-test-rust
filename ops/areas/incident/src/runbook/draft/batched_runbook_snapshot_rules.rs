use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched runbook snapshot must pass. Check returns its problems, or none.
pub trait BatchedRunbookSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRunbookSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSnapshotRequiredLabelRule {
    pub message: String,
}

impl BatchedRunbookSnapshotRule for BatchedRunbookSnapshotRequiredLabelRule {
    fn check(&self, item: &BatchedRunbookSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSnapshotRequiredCodeRule {
    pub message: String,
}

impl BatchedRunbookSnapshotRule for BatchedRunbookSnapshotRequiredCodeRule {
    fn check(&self, item: &BatchedRunbookSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSnapshotCodePrefixRule {
    pub prefix: String,
}

impl BatchedRunbookSnapshotRule for BatchedRunbookSnapshotCodePrefixRule {
    fn check(&self, item: &BatchedRunbookSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSnapshotAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BatchedRunbookSnapshotRule for BatchedRunbookSnapshotAllowedLabelRule {
    fn check(&self, item: &BatchedRunbookSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookSnapshotRule for BatchedRunbookSnapshotLabelLengthRule {
    fn check(&self, item: &BatchedRunbookSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched runbook snapshot.
#[derive(Debug, Clone)]
pub struct BatchedRunbookSnapshotValidator {
    pub rules: Vec<Rc<dyn BatchedRunbookSnapshotRule>>,
}

impl BatchedRunbookSnapshotValidator {
    pub fn validate(&self, item: &BatchedRunbookSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRunbookSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRunbookSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_runbook_snapshot_validator() -> BatchedRunbookSnapshotValidator {
    BatchedRunbookSnapshotValidator {
        rules: vec![
            Rc::new(BatchedRunbookSnapshotRequiredLabelRule {
                message: "batched runbook snapshot needs this field".to_string(),
            }) as Rc<dyn BatchedRunbookSnapshotRule>,
            Rc::new(BatchedRunbookSnapshotRequiredCodeRule {
                message: "batched runbook snapshot needs an entry".to_string(),
            }),
            Rc::new(BatchedRunbookSnapshotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BatchedRunbookSnapshotAllowedLabelRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(BatchedRunbookSnapshotLabelLengthRule { max_length: 1000 }),
        ],
    }
}
