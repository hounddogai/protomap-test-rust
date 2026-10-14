use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout cycle must pass. Check returns its problems, or none.
pub trait RolloutCycleRule: std::fmt::Debug {
    fn check(&self, item: &RolloutCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycleCodeLengthRule {
    pub max_length: i64,
}

impl RolloutCycleRule for RolloutCycleCodeLengthRule {
    fn check(&self, item: &RolloutCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycleAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl RolloutCycleRule for RolloutCycleAllowedSegmentRule {
    fn check(&self, item: &RolloutCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is above maximum", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycleTimezoneLengthRule {
    pub max_length: i64,
}

impl RolloutCycleRule for RolloutCycleTimezoneLengthRule {
    fn check(&self, item: &RolloutCycle) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycleRequiredCodeRule {
    pub message: String,
}

impl RolloutCycleRule for RolloutCycleRequiredCodeRule {
    fn check(&self, item: &RolloutCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycleSegmentLengthRule {
    pub max_length: i64,
}

impl RolloutCycleRule for RolloutCycleSegmentLengthRule {
    fn check(&self, item: &RolloutCycle) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycleLabelLengthRule {
    pub max_length: i64,
}

impl RolloutCycleRule for RolloutCycleLabelLengthRule {
    fn check(&self, item: &RolloutCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout cycle.
#[derive(Debug, Clone)]
pub struct RolloutCycleValidator {
    pub rules: Vec<Rc<dyn RolloutCycleRule>>,
}

impl RolloutCycleValidator {
    pub fn validate(&self, item: &RolloutCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_cycle_validator() -> RolloutCycleValidator {
    RolloutCycleValidator {
        rules: vec![
            Rc::new(RolloutCycleCodeLengthRule { max_length: 64 }) as Rc<dyn RolloutCycleRule>,
            Rc::new(RolloutCycleAllowedSegmentRule { allowed: Vec::new() }),
            Rc::new(RolloutCycleTimezoneLengthRule { max_length: 120 }),
            Rc::new(RolloutCycleRequiredCodeRule {
                message: "rollout cycle needs an entry".to_string(),
            }),
            Rc::new(RolloutCycleSegmentLengthRule { max_length: 32 }),
            Rc::new(RolloutCycleLabelLengthRule { max_length: 16 }),
        ],
    }
}
