use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy rollup must pass. Check returns its problems, or none.
pub trait TtlPolicyRollupRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRollupRequiredTitleRule {
    pub message: String,
}

impl TtlPolicyRollupRule for TtlPolicyRollupRequiredTitleRule {
    fn check(&self, item: &TtlPolicyRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRollupWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyRollupRule for TtlPolicyRollupWeightRangeRule {
    fn check(&self, item: &TtlPolicyRollup) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRollupRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyRollupRule for TtlPolicyRollupRequiredCodeRule {
    fn check(&self, item: &TtlPolicyRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRollupStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyRollupRule for TtlPolicyRollupStartsAtRangeRule {
    fn check(&self, item: &TtlPolicyRollup) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRollupRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyRollupRule for TtlPolicyRollupRetriesRangeRule {
    fn check(&self, item: &TtlPolicyRollup) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRollupRequiredSourceRule {
    pub message: String,
}

impl TtlPolicyRollupRule for TtlPolicyRollupRequiredSourceRule {
    fn check(&self, item: &TtlPolicyRollup) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy rollup.
#[derive(Debug, Clone)]
pub struct TtlPolicyRollupValidator {
    pub rules: Vec<Rc<dyn TtlPolicyRollupRule>>,
}

impl TtlPolicyRollupValidator {
    pub fn validate(&self, item: &TtlPolicyRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_rollup_validator() -> TtlPolicyRollupValidator {
    TtlPolicyRollupValidator {
        rules: vec![
            Rc::new(TtlPolicyRollupRequiredTitleRule {
                message: "ttl policy rollup needs this field".to_string(),
            }) as Rc<dyn TtlPolicyRollupRule>,
            Rc::new(TtlPolicyRollupWeightRangeRule { min: 6.8, max: 2183.0 }),
            Rc::new(TtlPolicyRollupRequiredCodeRule {
                message: "ttl policy rollup needs a value".to_string(),
            }),
            Rc::new(TtlPolicyRollupStartsAtRangeRule { min: 32, max: 1000 }),
            Rc::new(TtlPolicyRollupRetriesRangeRule { min: 64, max: 3600 }),
            Rc::new(TtlPolicyRollupRequiredSourceRule {
                message: "ttl policy rollup needs an entry".to_string(),
            }),
        ],
    }
}
