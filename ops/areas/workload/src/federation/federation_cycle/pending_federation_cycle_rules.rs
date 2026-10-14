use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending federation cycle must pass. Check returns its problems, or none.
pub trait PendingFederationCycleRule: std::fmt::Debug {
    fn check(&self, item: &PendingFederationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationCycleLabelLengthRule {
    pub max_length: i64,
}

impl PendingFederationCycleRule for PendingFederationCycleLabelLengthRule {
    fn check(&self, item: &PendingFederationCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationCycleRequiredCodeRule {
    pub message: String,
}

impl PendingFederationCycleRule for PendingFederationCycleRequiredCodeRule {
    fn check(&self, item: &PendingFederationCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationCycleCodeLengthRule {
    pub max_length: i64,
}

impl PendingFederationCycleRule for PendingFederationCycleCodeLengthRule {
    fn check(&self, item: &PendingFederationCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationCycleRequiredLabelRule {
    pub message: String,
}

impl PendingFederationCycleRule for PendingFederationCycleRequiredLabelRule {
    fn check(&self, item: &PendingFederationCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending federation cycle.
#[derive(Debug, Clone)]
pub struct PendingFederationCycleValidator {
    pub rules: Vec<Rc<dyn PendingFederationCycleRule>>,
}

impl PendingFederationCycleValidator {
    pub fn validate(&self, item: &PendingFederationCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingFederationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFederationCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_federation_cycle_validator() -> PendingFederationCycleValidator {
    PendingFederationCycleValidator {
        rules: vec![
            Rc::new(PendingFederationCycleLabelLengthRule { max_length: 16 }) as Rc<dyn PendingFederationCycleRule>,
            Rc::new(PendingFederationCycleRequiredCodeRule {
                message: "pending federation cycle needs an entry".to_string(),
            }),
            Rc::new(PendingFederationCycleCodeLengthRule { max_length: 1000 }),
            Rc::new(PendingFederationCycleRequiredLabelRule {
                message: "pending federation cycle needs this field".to_string(),
            }),
        ],
    }
}
