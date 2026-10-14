use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink score must pass. Check returns its problems, or none.
pub trait SinkScoreRule: std::fmt::Debug {
    fn check(&self, item: &SinkScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScoreRequiredCodeRule {
    pub message: String,
}

impl SinkScoreRule for SinkScoreRequiredCodeRule {
    fn check(&self, item: &SinkScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScoreRequiredExternalRefRule {
    pub message: String,
}

impl SinkScoreRule for SinkScoreRequiredExternalRefRule {
    fn check(&self, item: &SinkScore) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScoreCodePrefixRule {
    pub prefix: String,
}

impl SinkScoreRule for SinkScoreCodePrefixRule {
    fn check(&self, item: &SinkScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScoreMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkScoreRule for SinkScoreMarginRangeRule {
    fn check(&self, item: &SinkScore) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a sink score.
#[derive(Debug, Clone)]
pub struct SinkScoreValidator {
    pub rules: Vec<Rc<dyn SinkScoreRule>>,
}

impl SinkScoreValidator {
    pub fn validate(&self, item: &SinkScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_score_validator() -> SinkScoreValidator {
    SinkScoreValidator {
        rules: vec![
            Rc::new(SinkScoreRequiredCodeRule {
                message: "sink score needs an entry".to_string(),
            }) as Rc<dyn SinkScoreRule>,
            Rc::new(SinkScoreRequiredExternalRefRule {
                message: "sink score needs this field".to_string(),
            }),
            Rc::new(SinkScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SinkScoreMarginRangeRule { min: 6.6, max: 3596.7 }),
        ],
    }
}
