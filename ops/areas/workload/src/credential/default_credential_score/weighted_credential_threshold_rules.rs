use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted credential threshold must pass. Check returns its problems, or none.
pub trait WeightedCredentialThresholdRule: std::fmt::Debug {
    fn check(&self, item: &WeightedCredentialThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialThresholdSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedCredentialThresholdRule for WeightedCredentialThresholdSequenceRangeRule {
    fn check(&self, item: &WeightedCredentialThreshold) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialThresholdRequiredCipherRule {
    pub message: String,
}

impl WeightedCredentialThresholdRule for WeightedCredentialThresholdRequiredCipherRule {
    fn check(&self, item: &WeightedCredentialThreshold) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialThresholdRequiredSegmentRule {
    pub message: String,
}

impl WeightedCredentialThresholdRule for WeightedCredentialThresholdRequiredSegmentRule {
    fn check(&self, item: &WeightedCredentialThreshold) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialThresholdSegmentLengthRule {
    pub max_length: i64,
}

impl WeightedCredentialThresholdRule for WeightedCredentialThresholdSegmentLengthRule {
    fn check(&self, item: &WeightedCredentialThreshold) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted credential threshold.
#[derive(Debug, Clone)]
pub struct WeightedCredentialThresholdValidator {
    pub rules: Vec<Rc<dyn WeightedCredentialThresholdRule>>,
}

impl WeightedCredentialThresholdValidator {
    pub fn validate(&self, item: &WeightedCredentialThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedCredentialThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedCredentialThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_credential_threshold_validator() -> WeightedCredentialThresholdValidator {
    WeightedCredentialThresholdValidator {
        rules: vec![
            Rc::new(WeightedCredentialThresholdSequenceRangeRule { min: 32, max: 64 })
                as Rc<dyn WeightedCredentialThresholdRule>,
            Rc::new(WeightedCredentialThresholdRequiredCipherRule {
                message: "weighted credential threshold needs an entry".to_string(),
            }),
            Rc::new(WeightedCredentialThresholdRequiredSegmentRule {
                message: "weighted credential threshold needs a value".to_string(),
            }),
            Rc::new(WeightedCredentialThresholdSegmentLengthRule { max_length: 80 }),
        ],
    }
}
