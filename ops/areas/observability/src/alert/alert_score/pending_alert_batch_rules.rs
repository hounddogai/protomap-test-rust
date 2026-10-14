use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending alert batch must pass. Check returns its problems, or none.
pub trait PendingAlertBatchRule: std::fmt::Debug {
    fn check(&self, item: &PendingAlertBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAlertBatchAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PendingAlertBatchRule for PendingAlertBatchAllowedTitleRule {
    fn check(&self, item: &PendingAlertBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAlertBatchRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingAlertBatchRule for PendingAlertBatchRetriesRangeRule {
    fn check(&self, item: &PendingAlertBatch) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAlertBatchRequiredChannelRule {
    pub message: String,
}

impl PendingAlertBatchRule for PendingAlertBatchRequiredChannelRule {
    fn check(&self, item: &PendingAlertBatch) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAlertBatchRequiredTitleRule {
    pub message: String,
}

impl PendingAlertBatchRule for PendingAlertBatchRequiredTitleRule {
    fn check(&self, item: &PendingAlertBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAlertBatchSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingAlertBatchRule for PendingAlertBatchSequenceRangeRule {
    fn check(&self, item: &PendingAlertBatch) -> Vec<String> {
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

/// Runs every rule of a pending alert batch.
#[derive(Debug, Clone)]
pub struct PendingAlertBatchValidator {
    pub rules: Vec<Rc<dyn PendingAlertBatchRule>>,
}

impl PendingAlertBatchValidator {
    pub fn validate(&self, item: &PendingAlertBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingAlertBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingAlertBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_alert_batch_validator() -> PendingAlertBatchValidator {
    PendingAlertBatchValidator {
        rules: vec![
            Rc::new(PendingAlertBatchAllowedTitleRule {
                allowed: vec!["adaptive".to_string(), "regional".to_string(), "rolling".to_string()],
            }) as Rc<dyn PendingAlertBatchRule>,
            Rc::new(PendingAlertBatchRetriesRangeRule { min: 32, max: 255 }),
            Rc::new(PendingAlertBatchRequiredChannelRule {
                message: "pending alert batch needs a value".to_string(),
            }),
            Rc::new(PendingAlertBatchRequiredTitleRule {
                message: "pending alert batch needs an entry".to_string(),
            }),
            Rc::new(PendingAlertBatchSequenceRangeRule { min: 80, max: 255 }),
        ],
    }
}
