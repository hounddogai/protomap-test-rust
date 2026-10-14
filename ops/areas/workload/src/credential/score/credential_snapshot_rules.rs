use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential snapshot must pass. Check returns its problems, or none.
pub trait CredentialSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &CredentialSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotHostLengthRule {
    pub max_length: i64,
}

impl CredentialSnapshotRule for CredentialSnapshotHostLengthRule {
    fn check(&self, item: &CredentialSnapshot) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotRequiredLabelRule {
    pub message: String,
}

impl CredentialSnapshotRule for CredentialSnapshotRequiredLabelRule {
    fn check(&self, item: &CredentialSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl CredentialSnapshotRule for CredentialSnapshotAllowedLabelRule {
    fn check(&self, item: &CredentialSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialSnapshotRule for CredentialSnapshotRevisionRangeRule {
    fn check(&self, item: &CredentialSnapshot) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotCodePrefixRule {
    pub prefix: String,
}

impl CredentialSnapshotRule for CredentialSnapshotCodePrefixRule {
    fn check(&self, item: &CredentialSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotRequiredNamespaceRule {
    pub message: String,
}

impl CredentialSnapshotRule for CredentialSnapshotRequiredNamespaceRule {
    fn check(&self, item: &CredentialSnapshot) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential snapshot.
#[derive(Debug, Clone)]
pub struct CredentialSnapshotValidator {
    pub rules: Vec<Rc<dyn CredentialSnapshotRule>>,
}

impl CredentialSnapshotValidator {
    pub fn validate(&self, item: &CredentialSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_snapshot_validator() -> CredentialSnapshotValidator {
    CredentialSnapshotValidator {
        rules: vec![
            Rc::new(CredentialSnapshotHostLengthRule { max_length: 64 }) as Rc<dyn CredentialSnapshotRule>,
            Rc::new(CredentialSnapshotRequiredLabelRule {
                message: "credential snapshot needs a value".to_string(),
            }),
            Rc::new(CredentialSnapshotAllowedLabelRule {
                allowed: vec!["tiered".to_string(), "batched".to_string(), "tiered".to_string()],
            }),
            Rc::new(CredentialSnapshotRevisionRangeRule { min: 16, max: 32 }),
            Rc::new(CredentialSnapshotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CredentialSnapshotRequiredNamespaceRule {
                message: "credential snapshot needs this field".to_string(),
            }),
        ],
    }
}
