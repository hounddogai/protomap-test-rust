use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a escalation batch must pass. Check returns its problems, or none.
pub trait EscalationBatchRule: std::fmt::Debug {
    fn check(&self, item: &EscalationBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBatchWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationBatchRule for EscalationBatchWeightRangeRule {
    fn check(&self, item: &EscalationBatch) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBatchRequiredNameRule {
    pub message: String,
}

impl EscalationBatchRule for EscalationBatchRequiredNameRule {
    fn check(&self, item: &EscalationBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBatchAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl EscalationBatchRule for EscalationBatchAllowedSegmentRule {
    fn check(&self, item: &EscalationBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is inconsistent", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBatchSummaryLengthRule {
    pub max_length: i64,
}

impl EscalationBatchRule for EscalationBatchSummaryLengthRule {
    fn check(&self, item: &EscalationBatch) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation batch.
#[derive(Debug, Clone)]
pub struct EscalationBatchValidator {
    pub rules: Vec<Rc<dyn EscalationBatchRule>>,
}

impl EscalationBatchValidator {
    pub fn validate(&self, item: &EscalationBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_batch_validator() -> EscalationBatchValidator {
    EscalationBatchValidator {
        rules: vec![
            Rc::new(EscalationBatchWeightRangeRule { min: 1.7, max: 3678.2 }) as Rc<dyn EscalationBatchRule>,
            Rc::new(EscalationBatchRequiredNameRule {
                message: "escalation batch needs an entry".to_string(),
            }),
            Rc::new(EscalationBatchAllowedSegmentRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(EscalationBatchSummaryLengthRule { max_length: 86400 }),
        ],
    }
}
