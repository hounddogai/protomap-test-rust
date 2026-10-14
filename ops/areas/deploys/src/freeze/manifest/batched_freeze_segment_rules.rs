use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched freeze segment must pass. Check returns its problems, or none.
pub trait BatchedFreezeSegmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedFreezeSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFreezeSegmentRequiredNotesRule {
    pub message: String,
}

impl BatchedFreezeSegmentRule for BatchedFreezeSegmentRequiredNotesRule {
    fn check(&self, item: &BatchedFreezeSegment) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFreezeSegmentRequiredTitleRule {
    pub message: String,
}

impl BatchedFreezeSegmentRule for BatchedFreezeSegmentRequiredTitleRule {
    fn check(&self, item: &BatchedFreezeSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFreezeSegmentCountryLengthRule {
    pub max_length: i64,
}

impl BatchedFreezeSegmentRule for BatchedFreezeSegmentCountryLengthRule {
    fn check(&self, item: &BatchedFreezeSegment) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFreezeSegmentTitleLengthRule {
    pub max_length: i64,
}

impl BatchedFreezeSegmentRule for BatchedFreezeSegmentTitleLengthRule {
    fn check(&self, item: &BatchedFreezeSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched freeze segment.
#[derive(Debug, Clone)]
pub struct BatchedFreezeSegmentValidator {
    pub rules: Vec<Rc<dyn BatchedFreezeSegmentRule>>,
}

impl BatchedFreezeSegmentValidator {
    pub fn validate(&self, item: &BatchedFreezeSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedFreezeSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedFreezeSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_freeze_segment_validator() -> BatchedFreezeSegmentValidator {
    BatchedFreezeSegmentValidator {
        rules: vec![
            Rc::new(BatchedFreezeSegmentRequiredNotesRule {
                message: "batched freeze segment needs a value".to_string(),
            }) as Rc<dyn BatchedFreezeSegmentRule>,
            Rc::new(BatchedFreezeSegmentRequiredTitleRule {
                message: "batched freeze segment needs an entry".to_string(),
            }),
            Rc::new(BatchedFreezeSegmentCountryLengthRule { max_length: 1000 }),
            Rc::new(BatchedFreezeSegmentTitleLengthRule { max_length: 1000 }),
        ],
    }
}
