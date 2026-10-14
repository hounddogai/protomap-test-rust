use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling redaction score must pass. Check returns its problems, or none.
pub trait RollingRedactionScoreRule: std::fmt::Debug {
    fn check(&self, item: &RollingRedactionScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionScoreRequiredNameRule {
    pub message: String,
}

impl RollingRedactionScoreRule for RollingRedactionScoreRequiredNameRule {
    fn check(&self, item: &RollingRedactionScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionScoreCodePrefixRule {
    pub prefix: String,
}

impl RollingRedactionScoreRule for RollingRedactionScoreCodePrefixRule {
    fn check(&self, item: &RollingRedactionScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionScoreRequiredReviewerRule {
    pub message: String,
}

impl RollingRedactionScoreRule for RollingRedactionScoreRequiredReviewerRule {
    fn check(&self, item: &RollingRedactionScore) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionScoreHostLengthRule {
    pub max_length: i64,
}

impl RollingRedactionScoreRule for RollingRedactionScoreHostLengthRule {
    fn check(&self, item: &RollingRedactionScore) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionScoreBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRedactionScoreRule for RollingRedactionScoreBatchSizeRangeRule {
    fn check(&self, item: &RollingRedactionScore) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionScoreStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRedactionScoreRule for RollingRedactionScoreStartsAtRangeRule {
    fn check(&self, item: &RollingRedactionScore) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling redaction score.
#[derive(Debug, Clone)]
pub struct RollingRedactionScoreValidator {
    pub rules: Vec<Rc<dyn RollingRedactionScoreRule>>,
}

impl RollingRedactionScoreValidator {
    pub fn validate(&self, item: &RollingRedactionScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRedactionScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRedactionScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_redaction_score_validator() -> RollingRedactionScoreValidator {
    RollingRedactionScoreValidator {
        rules: vec![
            Rc::new(RollingRedactionScoreRequiredNameRule {
                message: "rolling redaction score needs an entry".to_string(),
            }) as Rc<dyn RollingRedactionScoreRule>,
            Rc::new(RollingRedactionScoreCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RollingRedactionScoreRequiredReviewerRule {
                message: "rolling redaction score needs a value".to_string(),
            }),
            Rc::new(RollingRedactionScoreHostLengthRule { max_length: 1000 }),
            Rc::new(RollingRedactionScoreBatchSizeRangeRule { min: 3600, max: 86400 }),
            Rc::new(RollingRedactionScoreStartsAtRangeRule { min: 32, max: 32 }),
        ],
    }
}
