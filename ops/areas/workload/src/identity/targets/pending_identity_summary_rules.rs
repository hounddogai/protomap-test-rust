use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending identity summary must pass. Check returns its problems, or none.
pub trait PendingIdentitySummaryRule: std::fmt::Debug {
    fn check(&self, item: &PendingIdentitySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentitySummarySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingIdentitySummaryRule for PendingIdentitySummarySequenceRangeRule {
    fn check(&self, item: &PendingIdentitySummary) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentitySummaryRequiredCodeRule {
    pub message: String,
}

impl PendingIdentitySummaryRule for PendingIdentitySummaryRequiredCodeRule {
    fn check(&self, item: &PendingIdentitySummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentitySummaryRequiredReviewerRule {
    pub message: String,
}

impl PendingIdentitySummaryRule for PendingIdentitySummaryRequiredReviewerRule {
    fn check(&self, item: &PendingIdentitySummary) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentitySummaryCodePrefixRule {
    pub prefix: String,
}

impl PendingIdentitySummaryRule for PendingIdentitySummaryCodePrefixRule {
    fn check(&self, item: &PendingIdentitySummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentitySummaryRequiredExternalRefRule {
    pub message: String,
}

impl PendingIdentitySummaryRule for PendingIdentitySummaryRequiredExternalRefRule {
    fn check(&self, item: &PendingIdentitySummary) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentitySummaryTitleLengthRule {
    pub max_length: i64,
}

impl PendingIdentitySummaryRule for PendingIdentitySummaryTitleLengthRule {
    fn check(&self, item: &PendingIdentitySummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending identity summary.
#[derive(Debug, Clone)]
pub struct PendingIdentitySummaryValidator {
    pub rules: Vec<Rc<dyn PendingIdentitySummaryRule>>,
}

impl PendingIdentitySummaryValidator {
    pub fn validate(&self, item: &PendingIdentitySummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingIdentitySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingIdentitySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_identity_summary_validator() -> PendingIdentitySummaryValidator {
    PendingIdentitySummaryValidator {
        rules: vec![
            Rc::new(PendingIdentitySummarySequenceRangeRule { min: 64, max: 3600 })
                as Rc<dyn PendingIdentitySummaryRule>,
            Rc::new(PendingIdentitySummaryRequiredCodeRule {
                message: "pending identity summary needs an entry".to_string(),
            }),
            Rc::new(PendingIdentitySummaryRequiredReviewerRule {
                message: "pending identity summary needs this field".to_string(),
            }),
            Rc::new(PendingIdentitySummaryCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingIdentitySummaryRequiredExternalRefRule {
                message: "pending identity summary needs a value".to_string(),
            }),
            Rc::new(PendingIdentitySummaryTitleLengthRule { max_length: 3600 }),
        ],
    }
}
