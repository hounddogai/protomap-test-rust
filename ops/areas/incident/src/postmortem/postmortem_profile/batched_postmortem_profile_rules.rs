use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched postmortem profile must pass. Check returns its problems, or none.
pub trait BatchedPostmortemProfileRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPostmortemProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProfileRequiredTitleRule {
    pub message: String,
}

impl BatchedPostmortemProfileRule for BatchedPostmortemProfileRequiredTitleRule {
    fn check(&self, item: &BatchedPostmortemProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProfileAllowedHostRule {
    pub allowed: Vec<String>,
}

impl BatchedPostmortemProfileRule for BatchedPostmortemProfileAllowedHostRule {
    fn check(&self, item: &BatchedPostmortemProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is out of range", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProfileCodePrefixRule {
    pub prefix: String,
}

impl BatchedPostmortemProfileRule for BatchedPostmortemProfileCodePrefixRule {
    fn check(&self, item: &BatchedPostmortemProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched postmortem profile.
#[derive(Debug, Clone)]
pub struct BatchedPostmortemProfileValidator {
    pub rules: Vec<Rc<dyn BatchedPostmortemProfileRule>>,
}

impl BatchedPostmortemProfileValidator {
    pub fn validate(&self, item: &BatchedPostmortemProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPostmortemProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPostmortemProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_postmortem_profile_validator() -> BatchedPostmortemProfileValidator {
    BatchedPostmortemProfileValidator {
        rules: vec![
            Rc::new(BatchedPostmortemProfileRequiredTitleRule {
                message: "batched postmortem profile needs a value".to_string(),
            }) as Rc<dyn BatchedPostmortemProfileRule>,
            Rc::new(BatchedPostmortemProfileAllowedHostRule { allowed: Vec::new() }),
            Rc::new(BatchedPostmortemProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
