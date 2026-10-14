use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched health check segment must pass. Check returns its problems, or none.
pub trait BatchedHealthCheckSegmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedHealthCheckSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckSegmentRequiredCodeRule {
    pub message: String,
}

impl BatchedHealthCheckSegmentRule for BatchedHealthCheckSegmentRequiredCodeRule {
    fn check(&self, item: &BatchedHealthCheckSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckSegmentRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedHealthCheckSegmentRule for BatchedHealthCheckSegmentRetriesRangeRule {
    fn check(&self, item: &BatchedHealthCheckSegment) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckSegmentAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedHealthCheckSegmentRule for BatchedHealthCheckSegmentAttemptsRangeRule {
    fn check(&self, item: &BatchedHealthCheckSegment) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckSegmentLabelLengthRule {
    pub max_length: i64,
}

impl BatchedHealthCheckSegmentRule for BatchedHealthCheckSegmentLabelLengthRule {
    fn check(&self, item: &BatchedHealthCheckSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckSegmentRequiredLabelRule {
    pub message: String,
}

impl BatchedHealthCheckSegmentRule for BatchedHealthCheckSegmentRequiredLabelRule {
    fn check(&self, item: &BatchedHealthCheckSegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched health check segment.
#[derive(Debug, Clone)]
pub struct BatchedHealthCheckSegmentValidator {
    pub rules: Vec<Rc<dyn BatchedHealthCheckSegmentRule>>,
}

impl BatchedHealthCheckSegmentValidator {
    pub fn validate(&self, item: &BatchedHealthCheckSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedHealthCheckSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedHealthCheckSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_health_check_segment_validator() -> BatchedHealthCheckSegmentValidator {
    BatchedHealthCheckSegmentValidator {
        rules: vec![
            Rc::new(BatchedHealthCheckSegmentRequiredCodeRule {
                message: "batched health check segment needs an entry".to_string(),
            }) as Rc<dyn BatchedHealthCheckSegmentRule>,
            Rc::new(BatchedHealthCheckSegmentRetriesRangeRule { min: 255, max: 1000 }),
            Rc::new(BatchedHealthCheckSegmentAttemptsRangeRule { min: 80, max: 80 }),
            Rc::new(BatchedHealthCheckSegmentLabelLengthRule { max_length: 86400 }),
            Rc::new(BatchedHealthCheckSegmentRequiredLabelRule {
                message: "batched health check segment needs this field".to_string(),
            }),
        ],
    }
}
