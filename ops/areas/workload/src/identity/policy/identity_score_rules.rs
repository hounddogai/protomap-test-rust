use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity score must pass. Check returns its problems, or none.
pub trait IdentityScoreRule: std::fmt::Debug {
    fn check(&self, item: &IdentityScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScoreRequiredSegmentRule {
    pub message: String,
}

impl IdentityScoreRule for IdentityScoreRequiredSegmentRule {
    fn check(&self, item: &IdentityScore) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScoreSegmentLengthRule {
    pub max_length: i64,
}

impl IdentityScoreRule for IdentityScoreSegmentLengthRule {
    fn check(&self, item: &IdentityScore) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScoreRequiredHostRule {
    pub message: String,
}

impl IdentityScoreRule for IdentityScoreRequiredHostRule {
    fn check(&self, item: &IdentityScore) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScoreHostLengthRule {
    pub max_length: i64,
}

impl IdentityScoreRule for IdentityScoreHostLengthRule {
    fn check(&self, item: &IdentityScore) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity score.
#[derive(Debug, Clone)]
pub struct IdentityScoreValidator {
    pub rules: Vec<Rc<dyn IdentityScoreRule>>,
}

impl IdentityScoreValidator {
    pub fn validate(&self, item: &IdentityScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_score_validator() -> IdentityScoreValidator {
    IdentityScoreValidator {
        rules: vec![
            Rc::new(IdentityScoreRequiredSegmentRule {
                message: "identity score needs a value".to_string(),
            }) as Rc<dyn IdentityScoreRule>,
            Rc::new(IdentityScoreSegmentLengthRule { max_length: 1000 }),
            Rc::new(IdentityScoreRequiredHostRule {
                message: "identity score needs an entry".to_string(),
            }),
            Rc::new(IdentityScoreHostLengthRule { max_length: 86400 }),
        ],
    }
}
