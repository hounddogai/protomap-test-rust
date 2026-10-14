use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy snapshot must pass. Check returns its problems, or none.
pub trait PolicySnapshotRule: std::fmt::Debug {
    fn check(&self, item: &PolicySnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySnapshotLabelLengthRule {
    pub max_length: i64,
}

impl PolicySnapshotRule for PolicySnapshotLabelLengthRule {
    fn check(&self, item: &PolicySnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySnapshotRequiredCodeRule {
    pub message: String,
}

impl PolicySnapshotRule for PolicySnapshotRequiredCodeRule {
    fn check(&self, item: &PolicySnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySnapshotCodeLengthRule {
    pub max_length: i64,
}

impl PolicySnapshotRule for PolicySnapshotCodeLengthRule {
    fn check(&self, item: &PolicySnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySnapshotCodePrefixRule {
    pub prefix: String,
}

impl PolicySnapshotRule for PolicySnapshotCodePrefixRule {
    fn check(&self, item: &PolicySnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySnapshotTenantIdLengthRule {
    pub max_length: i64,
}

impl PolicySnapshotRule for PolicySnapshotTenantIdLengthRule {
    fn check(&self, item: &PolicySnapshot) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy snapshot.
#[derive(Debug, Clone)]
pub struct PolicySnapshotValidator {
    pub rules: Vec<Rc<dyn PolicySnapshotRule>>,
}

impl PolicySnapshotValidator {
    pub fn validate(&self, item: &PolicySnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicySnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicySnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_snapshot_validator() -> PolicySnapshotValidator {
    PolicySnapshotValidator {
        rules: vec![
            Rc::new(PolicySnapshotLabelLengthRule { max_length: 3600 }) as Rc<dyn PolicySnapshotRule>,
            Rc::new(PolicySnapshotRequiredCodeRule {
                message: "policy snapshot needs an entry".to_string(),
            }),
            Rc::new(PolicySnapshotCodeLengthRule { max_length: 32 }),
            Rc::new(PolicySnapshotCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PolicySnapshotTenantIdLengthRule { max_length: 80 }),
        ],
    }
}
