use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal dns record checkpoint must pass. Check returns its problems, or none.
pub trait SeasonalDnsRecordCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDnsRecordCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpointArchivedGuardRule {
    pub limit: f64,
}

impl SeasonalDnsRecordCheckpointRule for SeasonalDnsRecordCheckpointArchivedGuardRule {
    fn check(&self, item: &SeasonalDnsRecordCheckpoint) -> Vec<String> {
        if item.archived && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while archived",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpointGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalDnsRecordCheckpointRule for SeasonalDnsRecordCheckpointGraceDaysRangeRule {
    fn check(&self, item: &SeasonalDnsRecordCheckpoint) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpointFingerprintLengthRule {
    pub max_length: i64,
}

impl SeasonalDnsRecordCheckpointRule for SeasonalDnsRecordCheckpointFingerprintLengthRule {
    fn check(&self, item: &SeasonalDnsRecordCheckpoint) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpointRequiredCodeRule {
    pub message: String,
}

impl SeasonalDnsRecordCheckpointRule for SeasonalDnsRecordCheckpointRequiredCodeRule {
    fn check(&self, item: &SeasonalDnsRecordCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpointRequiredFingerprintRule {
    pub message: String,
}

impl SeasonalDnsRecordCheckpointRule for SeasonalDnsRecordCheckpointRequiredFingerprintRule {
    fn check(&self, item: &SeasonalDnsRecordCheckpoint) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpointStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalDnsRecordCheckpointRule for SeasonalDnsRecordCheckpointStartsAtRangeRule {
    fn check(&self, item: &SeasonalDnsRecordCheckpoint) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal dns record checkpoint.
#[derive(Debug, Clone)]
pub struct SeasonalDnsRecordCheckpointValidator {
    pub rules: Vec<Rc<dyn SeasonalDnsRecordCheckpointRule>>,
}

impl SeasonalDnsRecordCheckpointValidator {
    pub fn validate(&self, item: &SeasonalDnsRecordCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalDnsRecordCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDnsRecordCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_dns_record_checkpoint_validator() -> SeasonalDnsRecordCheckpointValidator {
    SeasonalDnsRecordCheckpointValidator {
        rules: vec![
            Rc::new(SeasonalDnsRecordCheckpointArchivedGuardRule { limit: 0.9 })
                as Rc<dyn SeasonalDnsRecordCheckpointRule>,
            Rc::new(SeasonalDnsRecordCheckpointGraceDaysRangeRule { min: 255, max: 255 }),
            Rc::new(SeasonalDnsRecordCheckpointFingerprintLengthRule { max_length: 64 }),
            Rc::new(SeasonalDnsRecordCheckpointRequiredCodeRule {
                message: "seasonal dns record checkpoint needs an entry".to_string(),
            }),
            Rc::new(SeasonalDnsRecordCheckpointRequiredFingerprintRule {
                message: "seasonal dns record checkpoint needs an entry".to_string(),
            }),
            Rc::new(SeasonalDnsRecordCheckpointStartsAtRangeRule { min: 80, max: 255 }),
        ],
    }
}
