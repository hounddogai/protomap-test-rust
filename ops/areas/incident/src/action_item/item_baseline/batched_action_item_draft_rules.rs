use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched action item draft must pass. Check returns its problems, or none.
pub trait BatchedActionItemDraftRule: std::fmt::Debug {
    fn check(&self, item: &BatchedActionItemDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedActionItemDraftDescriptionLengthRule {
    pub max_length: i64,
}

impl BatchedActionItemDraftRule for BatchedActionItemDraftDescriptionLengthRule {
    fn check(&self, item: &BatchedActionItemDraft) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedActionItemDraftMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedActionItemDraftRule for BatchedActionItemDraftMarginRangeRule {
    fn check(&self, item: &BatchedActionItemDraft) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedActionItemDraftThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedActionItemDraftRule for BatchedActionItemDraftThresholdRangeRule {
    fn check(&self, item: &BatchedActionItemDraft) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched action item draft.
#[derive(Debug, Clone)]
pub struct BatchedActionItemDraftValidator {
    pub rules: Vec<Rc<dyn BatchedActionItemDraftRule>>,
}

impl BatchedActionItemDraftValidator {
    pub fn validate(&self, item: &BatchedActionItemDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedActionItemDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedActionItemDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_action_item_draft_validator() -> BatchedActionItemDraftValidator {
    BatchedActionItemDraftValidator {
        rules: vec![
            Rc::new(BatchedActionItemDraftDescriptionLengthRule { max_length: 80 })
                as Rc<dyn BatchedActionItemDraftRule>,
            Rc::new(BatchedActionItemDraftMarginRangeRule { min: 7.0, max: 3980.7 }),
            Rc::new(BatchedActionItemDraftThresholdRangeRule { min: 7.1, max: 4194.7 }),
        ],
    }
}
