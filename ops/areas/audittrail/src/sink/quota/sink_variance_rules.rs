use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink variance must pass. Check returns its problems, or none.
pub trait SinkVarianceRule: std::fmt::Debug {
    fn check(&self, item: &SinkVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkVarianceClusterLengthRule {
    pub max_length: i64,
}

impl SinkVarianceRule for SinkVarianceClusterLengthRule {
    fn check(&self, item: &SinkVariance) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkVarianceTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl SinkVarianceRule for SinkVarianceTagsLimitRule {
    fn check(&self, item: &SinkVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkVarianceRequiredTitleRule {
    pub message: String,
}

impl SinkVarianceRule for SinkVarianceRequiredTitleRule {
    fn check(&self, item: &SinkVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkVarianceRequiredClusterRule {
    pub message: String,
}

impl SinkVarianceRule for SinkVarianceRequiredClusterRule {
    fn check(&self, item: &SinkVariance) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink variance.
#[derive(Debug, Clone)]
pub struct SinkVarianceValidator {
    pub rules: Vec<Rc<dyn SinkVarianceRule>>,
}

impl SinkVarianceValidator {
    pub fn validate(&self, item: &SinkVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_variance_validator() -> SinkVarianceValidator {
    SinkVarianceValidator {
        rules: vec![
            Rc::new(SinkVarianceClusterLengthRule { max_length: 64 }) as Rc<dyn SinkVarianceRule>,
            Rc::new(SinkVarianceTagsLimitRule {
                max_count: 16,
                banned: vec!["lenient".to_string(), "manual".to_string()],
            }),
            Rc::new(SinkVarianceRequiredTitleRule {
                message: "sink variance needs a value".to_string(),
            }),
            Rc::new(SinkVarianceRequiredClusterRule {
                message: "sink variance needs an entry".to_string(),
            }),
        ],
    }
}
