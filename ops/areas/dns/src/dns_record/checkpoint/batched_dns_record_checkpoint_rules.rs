use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched dns record checkpoint must pass. Check returns its problems, or none.
pub trait BatchedDnsRecordCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &BatchedDnsRecordCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordCheckpointKeyIdLengthRule {
    pub max_length: i64,
}

impl BatchedDnsRecordCheckpointRule for BatchedDnsRecordCheckpointKeyIdLengthRule {
    fn check(&self, item: &BatchedDnsRecordCheckpoint) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl BatchedDnsRecordCheckpointRule for BatchedDnsRecordCheckpointLabelLengthRule {
    fn check(&self, item: &BatchedDnsRecordCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordCheckpointRequiredOwnerIdRule {
    pub message: String,
}

impl BatchedDnsRecordCheckpointRule for BatchedDnsRecordCheckpointRequiredOwnerIdRule {
    fn check(&self, item: &BatchedDnsRecordCheckpoint) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordCheckpointOwnerIdLengthRule {
    pub max_length: i64,
}

impl BatchedDnsRecordCheckpointRule for BatchedDnsRecordCheckpointOwnerIdLengthRule {
    fn check(&self, item: &BatchedDnsRecordCheckpoint) -> Vec<String> {
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

/// Runs every rule of a batched dns record checkpoint.
#[derive(Debug, Clone)]
pub struct BatchedDnsRecordCheckpointValidator {
    pub rules: Vec<Rc<dyn BatchedDnsRecordCheckpointRule>>,
}

impl BatchedDnsRecordCheckpointValidator {
    pub fn validate(&self, item: &BatchedDnsRecordCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedDnsRecordCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedDnsRecordCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_dns_record_checkpoint_validator() -> BatchedDnsRecordCheckpointValidator {
    BatchedDnsRecordCheckpointValidator {
        rules: vec![
            Rc::new(BatchedDnsRecordCheckpointKeyIdLengthRule { max_length: 80 })
                as Rc<dyn BatchedDnsRecordCheckpointRule>,
            Rc::new(BatchedDnsRecordCheckpointLabelLengthRule { max_length: 255 }),
            Rc::new(BatchedDnsRecordCheckpointRequiredOwnerIdRule {
                message: "batched dns record checkpoint needs this field".to_string(),
            }),
            Rc::new(BatchedDnsRecordCheckpointOwnerIdLengthRule { max_length: 32 }),
        ],
    }
}
