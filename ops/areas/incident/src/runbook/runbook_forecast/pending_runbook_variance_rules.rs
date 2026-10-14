use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending runbook variance must pass. Check returns its problems, or none.
pub trait PendingRunbookVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PendingRunbookVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookVarianceRequiredCodeRule {
    pub message: String,
}

impl PendingRunbookVarianceRule for PendingRunbookVarianceRequiredCodeRule {
    fn check(&self, item: &PendingRunbookVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookVarianceLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRunbookVarianceRule for PendingRunbookVarianceLimitRangeRule {
    fn check(&self, item: &PendingRunbookVariance) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookVarianceChannelLengthRule {
    pub max_length: i64,
}

impl PendingRunbookVarianceRule for PendingRunbookVarianceChannelLengthRule {
    fn check(&self, item: &PendingRunbookVariance) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookVarianceCodePrefixRule {
    pub prefix: String,
}

impl PendingRunbookVarianceRule for PendingRunbookVarianceCodePrefixRule {
    fn check(&self, item: &PendingRunbookVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookVarianceMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRunbookVarianceRule for PendingRunbookVarianceMinimumRangeRule {
    fn check(&self, item: &PendingRunbookVariance) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending runbook variance.
#[derive(Debug, Clone)]
pub struct PendingRunbookVarianceValidator {
    pub rules: Vec<Rc<dyn PendingRunbookVarianceRule>>,
}

impl PendingRunbookVarianceValidator {
    pub fn validate(&self, item: &PendingRunbookVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingRunbookVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRunbookVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_runbook_variance_validator() -> PendingRunbookVarianceValidator {
    PendingRunbookVarianceValidator {
        rules: vec![
            Rc::new(PendingRunbookVarianceRequiredCodeRule {
                message: "pending runbook variance needs an entry".to_string(),
            }) as Rc<dyn PendingRunbookVarianceRule>,
            Rc::new(PendingRunbookVarianceLimitRangeRule { min: 64, max: 80 }),
            Rc::new(PendingRunbookVarianceChannelLengthRule { max_length: 32 }),
            Rc::new(PendingRunbookVarianceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingRunbookVarianceMinimumRangeRule { min: 6.5, max: 421.6 }),
        ],
    }
}
