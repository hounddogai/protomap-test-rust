use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched policy quota must pass. Check returns its problems, or none.
pub trait BatchedPolicyQuotaRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicyQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyQuotaClusterLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyQuotaRule for BatchedPolicyQuotaClusterLengthRule {
    fn check(&self, item: &BatchedPolicyQuota) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyQuotaRequiredCodeRule {
    pub message: String,
}

impl BatchedPolicyQuotaRule for BatchedPolicyQuotaRequiredCodeRule {
    fn check(&self, item: &BatchedPolicyQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyQuotaWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPolicyQuotaRule for BatchedPolicyQuotaWindowDaysRangeRule {
    fn check(&self, item: &BatchedPolicyQuota) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched policy quota.
#[derive(Debug, Clone)]
pub struct BatchedPolicyQuotaValidator {
    pub rules: Vec<Rc<dyn BatchedPolicyQuotaRule>>,
}

impl BatchedPolicyQuotaValidator {
    pub fn validate(&self, item: &BatchedPolicyQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPolicyQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicyQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_quota_validator() -> BatchedPolicyQuotaValidator {
    BatchedPolicyQuotaValidator {
        rules: vec![
            Rc::new(BatchedPolicyQuotaClusterLengthRule { max_length: 255 }) as Rc<dyn BatchedPolicyQuotaRule>,
            Rc::new(BatchedPolicyQuotaRequiredCodeRule {
                message: "batched policy quota needs a value".to_string(),
            }),
            Rc::new(BatchedPolicyQuotaWindowDaysRangeRule { min: 32, max: 86400 }),
        ],
    }
}
