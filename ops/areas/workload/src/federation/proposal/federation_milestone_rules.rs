use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation milestone must pass. Check returns its problems, or none.
pub trait FederationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &FederationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMilestoneRequiredCodeRule {
    pub message: String,
}

impl FederationMilestoneRule for FederationMilestoneRequiredCodeRule {
    fn check(&self, item: &FederationMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMilestoneRequiredLabelRule {
    pub message: String,
}

impl FederationMilestoneRule for FederationMilestoneRequiredLabelRule {
    fn check(&self, item: &FederationMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMilestoneClusterLengthRule {
    pub max_length: i64,
}

impl FederationMilestoneRule for FederationMilestoneClusterLengthRule {
    fn check(&self, item: &FederationMilestone) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl FederationMilestoneRule for FederationMilestoneCodeLengthRule {
    fn check(&self, item: &FederationMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMilestoneRequiredFingerprintRule {
    pub message: String,
}

impl FederationMilestoneRule for FederationMilestoneRequiredFingerprintRule {
    fn check(&self, item: &FederationMilestone) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationMilestoneAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FederationMilestoneRule for FederationMilestoneAllowedLabelRule {
    fn check(&self, item: &FederationMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation milestone.
#[derive(Debug, Clone)]
pub struct FederationMilestoneValidator {
    pub rules: Vec<Rc<dyn FederationMilestoneRule>>,
}

impl FederationMilestoneValidator {
    pub fn validate(&self, item: &FederationMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_milestone_validator() -> FederationMilestoneValidator {
    FederationMilestoneValidator {
        rules: vec![
            Rc::new(FederationMilestoneRequiredCodeRule {
                message: "federation milestone needs an entry".to_string(),
            }) as Rc<dyn FederationMilestoneRule>,
            Rc::new(FederationMilestoneRequiredLabelRule {
                message: "federation milestone needs an entry".to_string(),
            }),
            Rc::new(FederationMilestoneClusterLengthRule { max_length: 255 }),
            Rc::new(FederationMilestoneCodeLengthRule { max_length: 32 }),
            Rc::new(FederationMilestoneRequiredFingerprintRule {
                message: "federation milestone needs a value".to_string(),
            }),
            Rc::new(FederationMilestoneAllowedLabelRule { allowed: Vec::new() }),
        ],
    }
}
