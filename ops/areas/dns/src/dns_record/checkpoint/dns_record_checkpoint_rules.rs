use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record checkpoint must pass. Check returns its problems, or none.
pub trait DnsRecordCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DnsRecordCheckpointRule for DnsRecordCheckpointAllowedCodeRule {
    fn check(&self, item: &DnsRecordCheckpoint) -> Vec<String> {
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
pub struct DnsRecordCheckpointCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordCheckpointRule for DnsRecordCheckpointCodePrefixRule {
    fn check(&self, item: &DnsRecordCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCheckpointDescriptionLengthRule {
    pub max_length: i64,
}

impl DnsRecordCheckpointRule for DnsRecordCheckpointDescriptionLengthRule {
    fn check(&self, item: &DnsRecordCheckpoint) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCheckpointRequiredNotesRule {
    pub message: String,
}

impl DnsRecordCheckpointRule for DnsRecordCheckpointRequiredNotesRule {
    fn check(&self, item: &DnsRecordCheckpoint) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record checkpoint.
#[derive(Debug, Clone)]
pub struct DnsRecordCheckpointValidator {
    pub rules: Vec<Rc<dyn DnsRecordCheckpointRule>>,
}

impl DnsRecordCheckpointValidator {
    pub fn validate(&self, item: &DnsRecordCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_checkpoint_validator() -> DnsRecordCheckpointValidator {
    DnsRecordCheckpointValidator {
        rules: vec![
            Rc::new(DnsRecordCheckpointAllowedCodeRule {
                allowed: vec!["default".to_string(), "global".to_string()],
            }) as Rc<dyn DnsRecordCheckpointRule>,
            Rc::new(DnsRecordCheckpointCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(DnsRecordCheckpointDescriptionLengthRule { max_length: 16 }),
            Rc::new(DnsRecordCheckpointRequiredNotesRule {
                message: "dns record checkpoint needs this field".to_string(),
            }),
        ],
    }
}
