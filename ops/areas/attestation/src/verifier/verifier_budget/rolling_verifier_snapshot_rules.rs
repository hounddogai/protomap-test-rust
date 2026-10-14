use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling verifier snapshot must pass. Check returns its problems, or none.
pub trait RollingVerifierSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RollingVerifierSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSnapshotNameLengthRule {
    pub max_length: i64,
}

impl RollingVerifierSnapshotRule for RollingVerifierSnapshotNameLengthRule {
    fn check(&self, item: &RollingVerifierSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSnapshotRequiredCodeRule {
    pub message: String,
}

impl RollingVerifierSnapshotRule for RollingVerifierSnapshotRequiredCodeRule {
    fn check(&self, item: &RollingVerifierSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSnapshotQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingVerifierSnapshotRule for RollingVerifierSnapshotQuotaUnitsRangeRule {
    fn check(&self, item: &RollingVerifierSnapshot) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling verifier snapshot.
#[derive(Debug, Clone)]
pub struct RollingVerifierSnapshotValidator {
    pub rules: Vec<Rc<dyn RollingVerifierSnapshotRule>>,
}

impl RollingVerifierSnapshotValidator {
    pub fn validate(&self, item: &RollingVerifierSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingVerifierSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingVerifierSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_verifier_snapshot_validator() -> RollingVerifierSnapshotValidator {
    RollingVerifierSnapshotValidator {
        rules: vec![
            Rc::new(RollingVerifierSnapshotNameLengthRule { max_length: 255 }) as Rc<dyn RollingVerifierSnapshotRule>,
            Rc::new(RollingVerifierSnapshotRequiredCodeRule {
                message: "rolling verifier snapshot needs a value".to_string(),
            }),
            Rc::new(RollingVerifierSnapshotQuotaUnitsRangeRule { min: 32, max: 32 }),
        ],
    }
}
