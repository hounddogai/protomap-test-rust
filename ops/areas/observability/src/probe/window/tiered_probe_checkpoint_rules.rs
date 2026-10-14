use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered probe checkpoint must pass. Check returns its problems, or none.
pub trait TieredProbeCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &TieredProbeCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeCheckpointRequiredCodeRule {
    pub message: String,
}

impl TieredProbeCheckpointRule for TieredProbeCheckpointRequiredCodeRule {
    fn check(&self, item: &TieredProbeCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeCheckpointApprovedGuardRule {
    pub limit: i64,
}

impl TieredProbeCheckpointRule for TieredProbeCheckpointApprovedGuardRule {
    fn check(&self, item: &TieredProbeCheckpoint) -> Vec<String> {
        if item.approved && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while approved",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeCheckpointNameLengthRule {
    pub max_length: i64,
}

impl TieredProbeCheckpointRule for TieredProbeCheckpointNameLengthRule {
    fn check(&self, item: &TieredProbeCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeCheckpointRequiredCurrencyRule {
    pub message: String,
}

impl TieredProbeCheckpointRule for TieredProbeCheckpointRequiredCurrencyRule {
    fn check(&self, item: &TieredProbeCheckpoint) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered probe checkpoint.
#[derive(Debug, Clone)]
pub struct TieredProbeCheckpointValidator {
    pub rules: Vec<Rc<dyn TieredProbeCheckpointRule>>,
}

impl TieredProbeCheckpointValidator {
    pub fn validate(&self, item: &TieredProbeCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredProbeCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredProbeCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_probe_checkpoint_validator() -> TieredProbeCheckpointValidator {
    TieredProbeCheckpointValidator {
        rules: vec![
            Rc::new(TieredProbeCheckpointRequiredCodeRule {
                message: "tiered probe checkpoint needs this field".to_string(),
            }) as Rc<dyn TieredProbeCheckpointRule>,
            Rc::new(TieredProbeCheckpointApprovedGuardRule { limit: 80 }),
            Rc::new(TieredProbeCheckpointNameLengthRule { max_length: 16 }),
            Rc::new(TieredProbeCheckpointRequiredCurrencyRule {
                message: "tiered probe checkpoint needs a value".to_string(),
            }),
        ],
    }
}
