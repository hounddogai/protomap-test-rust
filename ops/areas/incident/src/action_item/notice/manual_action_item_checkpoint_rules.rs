use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual action item checkpoint must pass. Check returns its problems, or none.
pub trait ManualActionItemCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ManualActionItemCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualActionItemCheckpointRequiredChecksumRule {
    pub message: String,
}

impl ManualActionItemCheckpointRule for ManualActionItemCheckpointRequiredChecksumRule {
    fn check(&self, item: &ManualActionItemCheckpoint) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualActionItemCheckpointRequiredCodeRule {
    pub message: String,
}

impl ManualActionItemCheckpointRule for ManualActionItemCheckpointRequiredCodeRule {
    fn check(&self, item: &ManualActionItemCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualActionItemCheckpointEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualActionItemCheckpointRule for ManualActionItemCheckpointEndsAtRangeRule {
    fn check(&self, item: &ManualActionItemCheckpoint) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualActionItemCheckpointTitleLengthRule {
    pub max_length: i64,
}

impl ManualActionItemCheckpointRule for ManualActionItemCheckpointTitleLengthRule {
    fn check(&self, item: &ManualActionItemCheckpoint) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualActionItemCheckpointWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualActionItemCheckpointRule for ManualActionItemCheckpointWeightRangeRule {
    fn check(&self, item: &ManualActionItemCheckpoint) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual action item checkpoint.
#[derive(Debug, Clone)]
pub struct ManualActionItemCheckpointValidator {
    pub rules: Vec<Rc<dyn ManualActionItemCheckpointRule>>,
}

impl ManualActionItemCheckpointValidator {
    pub fn validate(&self, item: &ManualActionItemCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualActionItemCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualActionItemCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_action_item_checkpoint_validator() -> ManualActionItemCheckpointValidator {
    ManualActionItemCheckpointValidator {
        rules: vec![
            Rc::new(ManualActionItemCheckpointRequiredChecksumRule {
                message: "manual action item checkpoint needs a value".to_string(),
            }) as Rc<dyn ManualActionItemCheckpointRule>,
            Rc::new(ManualActionItemCheckpointRequiredCodeRule {
                message: "manual action item checkpoint needs a value".to_string(),
            }),
            Rc::new(ManualActionItemCheckpointEndsAtRangeRule { min: 16, max: 86400 }),
            Rc::new(ManualActionItemCheckpointTitleLengthRule { max_length: 3600 }),
            Rc::new(ManualActionItemCheckpointWeightRangeRule { min: 6.6, max: 792.1 }),
        ],
    }
}
