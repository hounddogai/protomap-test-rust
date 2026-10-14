use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending retention segment must pass. Check returns its problems, or none.
pub trait PendingRetentionSegmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingRetentionSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionSegmentRequiredNameRule {
    pub message: String,
}

impl PendingRetentionSegmentRule for PendingRetentionSegmentRequiredNameRule {
    fn check(&self, item: &PendingRetentionSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionSegmentRequiredCodeRule {
    pub message: String,
}

impl PendingRetentionSegmentRule for PendingRetentionSegmentRequiredCodeRule {
    fn check(&self, item: &PendingRetentionSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionSegmentCodePrefixRule {
    pub prefix: String,
}

impl PendingRetentionSegmentRule for PendingRetentionSegmentCodePrefixRule {
    fn check(&self, item: &PendingRetentionSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionSegmentRequiredHostRule {
    pub message: String,
}

impl PendingRetentionSegmentRule for PendingRetentionSegmentRequiredHostRule {
    fn check(&self, item: &PendingRetentionSegment) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending retention segment.
#[derive(Debug, Clone)]
pub struct PendingRetentionSegmentValidator {
    pub rules: Vec<Rc<dyn PendingRetentionSegmentRule>>,
}

impl PendingRetentionSegmentValidator {
    pub fn validate(&self, item: &PendingRetentionSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRetentionSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRetentionSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_retention_segment_validator() -> PendingRetentionSegmentValidator {
    PendingRetentionSegmentValidator {
        rules: vec![
            Rc::new(PendingRetentionSegmentRequiredNameRule {
                message: "pending retention segment needs an entry".to_string(),
            }) as Rc<dyn PendingRetentionSegmentRule>,
            Rc::new(PendingRetentionSegmentRequiredCodeRule {
                message: "pending retention segment needs this field".to_string(),
            }),
            Rc::new(PendingRetentionSegmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PendingRetentionSegmentRequiredHostRule {
                message: "pending retention segment needs an entry".to_string(),
            }),
        ],
    }
}
