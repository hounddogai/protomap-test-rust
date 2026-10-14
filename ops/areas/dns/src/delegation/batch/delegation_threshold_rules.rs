use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation threshold must pass. Check returns its problems, or none.
pub trait DelegationThresholdRule: std::fmt::Debug {
    fn check(&self, item: &DelegationThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationThresholdNameLengthRule {
    pub max_length: i64,
}

impl DelegationThresholdRule for DelegationThresholdNameLengthRule {
    fn check(&self, item: &DelegationThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationThresholdClusterLengthRule {
    pub max_length: i64,
}

impl DelegationThresholdRule for DelegationThresholdClusterLengthRule {
    fn check(&self, item: &DelegationThreshold) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationThresholdLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationThresholdRule for DelegationThresholdLimitRangeRule {
    fn check(&self, item: &DelegationThreshold) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation threshold.
#[derive(Debug, Clone)]
pub struct DelegationThresholdValidator {
    pub rules: Vec<Rc<dyn DelegationThresholdRule>>,
}

impl DelegationThresholdValidator {
    pub fn validate(&self, item: &DelegationThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_threshold_validator() -> DelegationThresholdValidator {
    DelegationThresholdValidator {
        rules: vec![
            Rc::new(DelegationThresholdNameLengthRule { max_length: 3600 }) as Rc<dyn DelegationThresholdRule>,
            Rc::new(DelegationThresholdClusterLengthRule { max_length: 255 }),
            Rc::new(DelegationThresholdLimitRangeRule { min: 16, max: 3600 }),
        ],
    }
}
