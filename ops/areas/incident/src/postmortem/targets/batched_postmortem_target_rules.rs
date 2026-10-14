use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched postmortem target must pass. Check returns its problems, or none.
pub trait BatchedPostmortemTargetRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPostmortemTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemTargetRequiredExternalRefRule {
    pub message: String,
}

impl BatchedPostmortemTargetRule for BatchedPostmortemTargetRequiredExternalRefRule {
    fn check(&self, item: &BatchedPostmortemTarget) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemTargetPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPostmortemTargetRule for BatchedPostmortemTargetPriorityRangeRule {
    fn check(&self, item: &BatchedPostmortemTarget) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemTargetAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BatchedPostmortemTargetRule for BatchedPostmortemTargetAllowedLabelRule {
    fn check(&self, item: &BatchedPostmortemTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemTargetCodeLengthRule {
    pub max_length: i64,
}

impl BatchedPostmortemTargetRule for BatchedPostmortemTargetCodeLengthRule {
    fn check(&self, item: &BatchedPostmortemTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched postmortem target.
#[derive(Debug, Clone)]
pub struct BatchedPostmortemTargetValidator {
    pub rules: Vec<Rc<dyn BatchedPostmortemTargetRule>>,
}

impl BatchedPostmortemTargetValidator {
    pub fn validate(&self, item: &BatchedPostmortemTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPostmortemTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPostmortemTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_postmortem_target_validator() -> BatchedPostmortemTargetValidator {
    BatchedPostmortemTargetValidator {
        rules: vec![
            Rc::new(BatchedPostmortemTargetRequiredExternalRefRule {
                message: "batched postmortem target needs a value".to_string(),
            }) as Rc<dyn BatchedPostmortemTargetRule>,
            Rc::new(BatchedPostmortemTargetPriorityRangeRule { min: 64, max: 3600 }),
            Rc::new(BatchedPostmortemTargetAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(BatchedPostmortemTargetCodeLengthRule { max_length: 255 }),
        ],
    }
}
