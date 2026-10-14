use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover snapshot must pass. Check returns its problems, or none.
pub trait FailoverSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &FailoverSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSnapshotScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverSnapshotRule for FailoverSnapshotScoreRangeRule {
    fn check(&self, item: &FailoverSnapshot) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSnapshotRequiredFingerprintRule {
    pub message: String,
}

impl FailoverSnapshotRule for FailoverSnapshotRequiredFingerprintRule {
    fn check(&self, item: &FailoverSnapshot) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSnapshotRequiredTitleRule {
    pub message: String,
}

impl FailoverSnapshotRule for FailoverSnapshotRequiredTitleRule {
    fn check(&self, item: &FailoverSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSnapshotCodePrefixRule {
    pub prefix: String,
}

impl FailoverSnapshotRule for FailoverSnapshotCodePrefixRule {
    fn check(&self, item: &FailoverSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSnapshotRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverSnapshotRule for FailoverSnapshotRetriesRangeRule {
    fn check(&self, item: &FailoverSnapshot) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl FailoverSnapshotRule for FailoverSnapshotCodeLengthRule {
    fn check(&self, item: &FailoverSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover snapshot.
#[derive(Debug, Clone)]
pub struct FailoverSnapshotValidator {
    pub rules: Vec<Rc<dyn FailoverSnapshotRule>>,
}

impl FailoverSnapshotValidator {
    pub fn validate(&self, item: &FailoverSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_snapshot_validator() -> FailoverSnapshotValidator {
    FailoverSnapshotValidator {
        rules: vec![
            Rc::new(FailoverSnapshotScoreRangeRule { min: 2.3, max: 4149.2 }) as Rc<dyn FailoverSnapshotRule>,
            Rc::new(FailoverSnapshotRequiredFingerprintRule {
                message: "failover snapshot needs this field".to_string(),
            }),
            Rc::new(FailoverSnapshotRequiredTitleRule {
                message: "failover snapshot needs a value".to_string(),
            }),
            Rc::new(FailoverSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(FailoverSnapshotRetriesRangeRule { min: 255, max: 3600 }),
            Rc::new(FailoverSnapshotCodeLengthRule { max_length: 80 }),
        ],
    }
}
