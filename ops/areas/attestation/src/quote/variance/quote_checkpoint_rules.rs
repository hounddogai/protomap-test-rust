use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote checkpoint must pass. Check returns its problems, or none.
pub trait QuoteCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &QuoteCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCheckpointReviewerLengthRule {
    pub max_length: i64,
}

impl QuoteCheckpointRule for QuoteCheckpointReviewerLengthRule {
    fn check(&self, item: &QuoteCheckpoint) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl QuoteCheckpointRule for QuoteCheckpointLabelLengthRule {
    fn check(&self, item: &QuoteCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCheckpointCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteCheckpointRule for QuoteCheckpointCeilingRangeRule {
    fn check(&self, item: &QuoteCheckpoint) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteCheckpointSourceLengthRule {
    pub max_length: i64,
}

impl QuoteCheckpointRule for QuoteCheckpointSourceLengthRule {
    fn check(&self, item: &QuoteCheckpoint) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote checkpoint.
#[derive(Debug, Clone)]
pub struct QuoteCheckpointValidator {
    pub rules: Vec<Rc<dyn QuoteCheckpointRule>>,
}

impl QuoteCheckpointValidator {
    pub fn validate(&self, item: &QuoteCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_checkpoint_validator() -> QuoteCheckpointValidator {
    QuoteCheckpointValidator {
        rules: vec![
            Rc::new(QuoteCheckpointReviewerLengthRule { max_length: 32 }) as Rc<dyn QuoteCheckpointRule>,
            Rc::new(QuoteCheckpointLabelLengthRule { max_length: 255 }),
            Rc::new(QuoteCheckpointCeilingRangeRule { min: 5.5, max: 1097.5 }),
            Rc::new(QuoteCheckpointSourceLengthRule { max_length: 80 }),
        ],
    }
}
