use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default retention checkpoint must pass. Check returns its problems, or none.
pub trait DefaultRetentionCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRetentionCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionCheckpointRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRetentionCheckpointRule for DefaultRetentionCheckpointRevisionRangeRule {
    fn check(&self, item: &DefaultRetentionCheckpoint) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionCheckpointQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRetentionCheckpointRule for DefaultRetentionCheckpointQuantityRangeRule {
    fn check(&self, item: &DefaultRetentionCheckpoint) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionCheckpointLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRetentionCheckpointRule for DefaultRetentionCheckpointLimitRangeRule {
    fn check(&self, item: &DefaultRetentionCheckpoint) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionCheckpointRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRetentionCheckpointRule for DefaultRetentionCheckpointRatioRangeRule {
    fn check(&self, item: &DefaultRetentionCheckpoint) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default retention checkpoint.
#[derive(Debug, Clone)]
pub struct DefaultRetentionCheckpointValidator {
    pub rules: Vec<Rc<dyn DefaultRetentionCheckpointRule>>,
}

impl DefaultRetentionCheckpointValidator {
    pub fn validate(&self, item: &DefaultRetentionCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRetentionCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRetentionCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_retention_checkpoint_validator() -> DefaultRetentionCheckpointValidator {
    DefaultRetentionCheckpointValidator {
        rules: vec![
            Rc::new(DefaultRetentionCheckpointRevisionRangeRule { min: 80, max: 86400 })
                as Rc<dyn DefaultRetentionCheckpointRule>,
            Rc::new(DefaultRetentionCheckpointQuantityRangeRule { min: 32, max: 64 }),
            Rc::new(DefaultRetentionCheckpointLimitRangeRule { min: 3600, max: 3600 }),
            Rc::new(DefaultRetentionCheckpointRatioRangeRule { min: 3.1, max: 4362.7 }),
        ],
    }
}
