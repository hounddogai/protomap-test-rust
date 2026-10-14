use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual runbook checkpoint must pass. Check returns its problems, or none.
pub trait ManualRunbookCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ManualRunbookCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookCheckpointCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualRunbookCheckpointRule for ManualRunbookCheckpointCeilingRangeRule {
    fn check(&self, item: &ManualRunbookCheckpoint) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookCheckpointCodePrefixRule {
    pub prefix: String,
}

impl ManualRunbookCheckpointRule for ManualRunbookCheckpointCodePrefixRule {
    fn check(&self, item: &ManualRunbookCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookCheckpointAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ManualRunbookCheckpointRule for ManualRunbookCheckpointAllowedNameRule {
    fn check(&self, item: &ManualRunbookCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookCheckpointRequiredNameRule {
    pub message: String,
}

impl ManualRunbookCheckpointRule for ManualRunbookCheckpointRequiredNameRule {
    fn check(&self, item: &ManualRunbookCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookCheckpointScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualRunbookCheckpointRule for ManualRunbookCheckpointScoreRangeRule {
    fn check(&self, item: &ManualRunbookCheckpoint) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual runbook checkpoint.
#[derive(Debug, Clone)]
pub struct ManualRunbookCheckpointValidator {
    pub rules: Vec<Rc<dyn ManualRunbookCheckpointRule>>,
}

impl ManualRunbookCheckpointValidator {
    pub fn validate(&self, item: &ManualRunbookCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRunbookCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRunbookCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_runbook_checkpoint_validator() -> ManualRunbookCheckpointValidator {
    ManualRunbookCheckpointValidator {
        rules: vec![
            Rc::new(ManualRunbookCheckpointCeilingRangeRule { min: 7.2, max: 3407.0 })
                as Rc<dyn ManualRunbookCheckpointRule>,
            Rc::new(ManualRunbookCheckpointCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ManualRunbookCheckpointAllowedNameRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(ManualRunbookCheckpointRequiredNameRule {
                message: "manual runbook checkpoint needs an entry".to_string(),
            }),
            Rc::new(ManualRunbookCheckpointScoreRangeRule { min: 4.1, max: 888.5 }),
        ],
    }
}
