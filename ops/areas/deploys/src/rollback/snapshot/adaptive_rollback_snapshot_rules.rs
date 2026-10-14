use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive rollback snapshot must pass. Check returns its problems, or none.
pub trait AdaptiveRollbackSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRollbackSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackSnapshotWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveRollbackSnapshotRule for AdaptiveRollbackSnapshotWindowDaysRangeRule {
    fn check(&self, item: &AdaptiveRollbackSnapshot) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackSnapshotRequiredCodeRule {
    pub message: String,
}

impl AdaptiveRollbackSnapshotRule for AdaptiveRollbackSnapshotRequiredCodeRule {
    fn check(&self, item: &AdaptiveRollbackSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackSnapshotNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveRollbackSnapshotRule for AdaptiveRollbackSnapshotNameLengthRule {
    fn check(&self, item: &AdaptiveRollbackSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackSnapshotRequiredNameRule {
    pub message: String,
}

impl AdaptiveRollbackSnapshotRule for AdaptiveRollbackSnapshotRequiredNameRule {
    fn check(&self, item: &AdaptiveRollbackSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive rollback snapshot.
#[derive(Debug, Clone)]
pub struct AdaptiveRollbackSnapshotValidator {
    pub rules: Vec<Rc<dyn AdaptiveRollbackSnapshotRule>>,
}

impl AdaptiveRollbackSnapshotValidator {
    pub fn validate(&self, item: &AdaptiveRollbackSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveRollbackSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRollbackSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rollback_snapshot_validator() -> AdaptiveRollbackSnapshotValidator {
    AdaptiveRollbackSnapshotValidator {
        rules: vec![
            Rc::new(AdaptiveRollbackSnapshotWindowDaysRangeRule { min: 80, max: 255 })
                as Rc<dyn AdaptiveRollbackSnapshotRule>,
            Rc::new(AdaptiveRollbackSnapshotRequiredCodeRule {
                message: "adaptive rollback snapshot needs an entry".to_string(),
            }),
            Rc::new(AdaptiveRollbackSnapshotNameLengthRule { max_length: 255 }),
            Rc::new(AdaptiveRollbackSnapshotRequiredNameRule {
                message: "adaptive rollback snapshot needs a value".to_string(),
            }),
        ],
    }
}
