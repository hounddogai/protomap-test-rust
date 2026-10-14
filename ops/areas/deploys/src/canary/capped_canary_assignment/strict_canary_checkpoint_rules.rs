use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict canary checkpoint must pass. Check returns its problems, or none.
pub trait StrictCanaryCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &StrictCanaryCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl StrictCanaryCheckpointRule for StrictCanaryCheckpointAllowedCodeRule {
    fn check(&self, item: &StrictCanaryCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryCheckpointCodePrefixRule {
    pub prefix: String,
}

impl StrictCanaryCheckpointRule for StrictCanaryCheckpointCodePrefixRule {
    fn check(&self, item: &StrictCanaryCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryCheckpointRequiredNameRule {
    pub message: String,
}

impl StrictCanaryCheckpointRule for StrictCanaryCheckpointRequiredNameRule {
    fn check(&self, item: &StrictCanaryCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryCheckpointCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictCanaryCheckpointRule for StrictCanaryCheckpointCeilingRangeRule {
    fn check(&self, item: &StrictCanaryCheckpoint) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict canary checkpoint.
#[derive(Debug, Clone)]
pub struct StrictCanaryCheckpointValidator {
    pub rules: Vec<Rc<dyn StrictCanaryCheckpointRule>>,
}

impl StrictCanaryCheckpointValidator {
    pub fn validate(&self, item: &StrictCanaryCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictCanaryCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictCanaryCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_canary_checkpoint_validator() -> StrictCanaryCheckpointValidator {
    StrictCanaryCheckpointValidator {
        rules: vec![
            Rc::new(StrictCanaryCheckpointAllowedCodeRule { allowed: Vec::new() })
                as Rc<dyn StrictCanaryCheckpointRule>,
            Rc::new(StrictCanaryCheckpointCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(StrictCanaryCheckpointRequiredNameRule {
                message: "strict canary checkpoint needs an entry".to_string(),
            }),
            Rc::new(StrictCanaryCheckpointCeilingRangeRule { min: 7.2, max: 4757.9 }),
        ],
    }
}
