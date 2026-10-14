use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling credential snapshot must pass. Check returns its problems, or none.
pub trait RollingCredentialSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RollingCredentialSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSnapshotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingCredentialSnapshotRule for RollingCredentialSnapshotUpdatedAtRangeRule {
    fn check(&self, item: &RollingCredentialSnapshot) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSnapshotPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingCredentialSnapshotRule for RollingCredentialSnapshotPriorityRangeRule {
    fn check(&self, item: &RollingCredentialSnapshot) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSnapshotRequiredLabelRule {
    pub message: String,
}

impl RollingCredentialSnapshotRule for RollingCredentialSnapshotRequiredLabelRule {
    fn check(&self, item: &RollingCredentialSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSnapshotIssuerLengthRule {
    pub max_length: i64,
}

impl RollingCredentialSnapshotRule for RollingCredentialSnapshotIssuerLengthRule {
    fn check(&self, item: &RollingCredentialSnapshot) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSnapshotRequiredCodeRule {
    pub message: String,
}

impl RollingCredentialSnapshotRule for RollingCredentialSnapshotRequiredCodeRule {
    fn check(&self, item: &RollingCredentialSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSnapshotRequiredExternalRefRule {
    pub message: String,
}

impl RollingCredentialSnapshotRule for RollingCredentialSnapshotRequiredExternalRefRule {
    fn check(&self, item: &RollingCredentialSnapshot) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling credential snapshot.
#[derive(Debug, Clone)]
pub struct RollingCredentialSnapshotValidator {
    pub rules: Vec<Rc<dyn RollingCredentialSnapshotRule>>,
}

impl RollingCredentialSnapshotValidator {
    pub fn validate(&self, item: &RollingCredentialSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingCredentialSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingCredentialSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_credential_snapshot_validator() -> RollingCredentialSnapshotValidator {
    RollingCredentialSnapshotValidator {
        rules: vec![
            Rc::new(RollingCredentialSnapshotUpdatedAtRangeRule { min: 16, max: 255 })
                as Rc<dyn RollingCredentialSnapshotRule>,
            Rc::new(RollingCredentialSnapshotPriorityRangeRule { min: 3600, max: 3600 }),
            Rc::new(RollingCredentialSnapshotRequiredLabelRule {
                message: "rolling credential snapshot needs an entry".to_string(),
            }),
            Rc::new(RollingCredentialSnapshotIssuerLengthRule { max_length: 80 }),
            Rc::new(RollingCredentialSnapshotRequiredCodeRule {
                message: "rolling credential snapshot needs a value".to_string(),
            }),
            Rc::new(RollingCredentialSnapshotRequiredExternalRefRule {
                message: "rolling credential snapshot needs a value".to_string(),
            }),
        ],
    }
}
