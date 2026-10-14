use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering record must pass. Check returns its problems, or none.
pub trait PeeringRecordRule: std::fmt::Debug {
    fn check(&self, item: &PeeringRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecordAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringRecordRule for PeeringRecordAttemptsRangeRule {
    fn check(&self, item: &PeeringRecord) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecordPrincipalLengthRule {
    pub max_length: i64,
}

impl PeeringRecordRule for PeeringRecordPrincipalLengthRule {
    fn check(&self, item: &PeeringRecord) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecordRequiredPrincipalRule {
    pub message: String,
}

impl PeeringRecordRule for PeeringRecordRequiredPrincipalRule {
    fn check(&self, item: &PeeringRecord) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecordRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringRecordRule for PeeringRecordRetriesRangeRule {
    fn check(&self, item: &PeeringRecord) -> Vec<String> {
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
pub struct PeeringRecordRequiredFingerprintRule {
    pub message: String,
}

impl PeeringRecordRule for PeeringRecordRequiredFingerprintRule {
    fn check(&self, item: &PeeringRecord) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecordQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringRecordRule for PeeringRecordQuantityRangeRule {
    fn check(&self, item: &PeeringRecord) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a peering record.
#[derive(Debug, Clone)]
pub struct PeeringRecordValidator {
    pub rules: Vec<Rc<dyn PeeringRecordRule>>,
}

impl PeeringRecordValidator {
    pub fn validate(&self, item: &PeeringRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_record_validator() -> PeeringRecordValidator {
    PeeringRecordValidator {
        rules: vec![
            Rc::new(PeeringRecordAttemptsRangeRule { min: 16, max: 80 }) as Rc<dyn PeeringRecordRule>,
            Rc::new(PeeringRecordPrincipalLengthRule { max_length: 3600 }),
            Rc::new(PeeringRecordRequiredPrincipalRule {
                message: "peering record needs this field".to_string(),
            }),
            Rc::new(PeeringRecordRetriesRangeRule { min: 32, max: 3600 }),
            Rc::new(PeeringRecordRequiredFingerprintRule {
                message: "peering record needs a value".to_string(),
            }),
            Rc::new(PeeringRecordQuantityRangeRule { min: 16, max: 32 }),
        ],
    }
}
