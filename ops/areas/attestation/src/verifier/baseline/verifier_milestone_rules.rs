use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier milestone must pass. Check returns its problems, or none.
pub trait VerifierMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &VerifierMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestoneRequiredTitleRule {
    pub message: String,
}

impl VerifierMilestoneRule for VerifierMilestoneRequiredTitleRule {
    fn check(&self, item: &VerifierMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestoneRequiredExternalRefRule {
    pub message: String,
}

impl VerifierMilestoneRule for VerifierMilestoneRequiredExternalRefRule {
    fn check(&self, item: &VerifierMilestone) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestoneRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierMilestoneRule for VerifierMilestoneRetriesRangeRule {
    fn check(&self, item: &VerifierMilestone) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestoneExternalRefLengthRule {
    pub max_length: i64,
}

impl VerifierMilestoneRule for VerifierMilestoneExternalRefLengthRule {
    fn check(&self, item: &VerifierMilestone) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestoneRequiredCodeRule {
    pub message: String,
}

impl VerifierMilestoneRule for VerifierMilestoneRequiredCodeRule {
    fn check(&self, item: &VerifierMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestoneAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl VerifierMilestoneRule for VerifierMilestoneAllowedTitleRule {
    fn check(&self, item: &VerifierMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier milestone.
#[derive(Debug, Clone)]
pub struct VerifierMilestoneValidator {
    pub rules: Vec<Rc<dyn VerifierMilestoneRule>>,
}

impl VerifierMilestoneValidator {
    pub fn validate(&self, item: &VerifierMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_milestone_validator() -> VerifierMilestoneValidator {
    VerifierMilestoneValidator {
        rules: vec![
            Rc::new(VerifierMilestoneRequiredTitleRule {
                message: "verifier milestone needs an entry".to_string(),
            }) as Rc<dyn VerifierMilestoneRule>,
            Rc::new(VerifierMilestoneRequiredExternalRefRule {
                message: "verifier milestone needs a value".to_string(),
            }),
            Rc::new(VerifierMilestoneRetriesRangeRule { min: 120, max: 1000 }),
            Rc::new(VerifierMilestoneExternalRefLengthRule { max_length: 3600 }),
            Rc::new(VerifierMilestoneRequiredCodeRule {
                message: "verifier milestone needs this field".to_string(),
            }),
            Rc::new(VerifierMilestoneAllowedTitleRule {
                allowed: vec!["tiered".to_string(), "lenient".to_string(), "adaptive".to_string()],
            }),
        ],
    }
}
