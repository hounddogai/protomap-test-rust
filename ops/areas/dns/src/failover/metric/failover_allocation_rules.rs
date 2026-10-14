use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover allocation must pass. Check returns its problems, or none.
pub trait FailoverAllocationRule: std::fmt::Debug {
    fn check(&self, item: &FailoverAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAllocationRegionLengthRule {
    pub max_length: i64,
}

impl FailoverAllocationRule for FailoverAllocationRegionLengthRule {
    fn check(&self, item: &FailoverAllocation) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAllocationCurrencyLengthRule {
    pub max_length: i64,
}

impl FailoverAllocationRule for FailoverAllocationCurrencyLengthRule {
    fn check(&self, item: &FailoverAllocation) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAllocationRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverAllocationRule for FailoverAllocationRevisionRangeRule {
    fn check(&self, item: &FailoverAllocation) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAllocationExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverAllocationRule for FailoverAllocationExpiresAtRangeRule {
    fn check(&self, item: &FailoverAllocation) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover allocation.
#[derive(Debug, Clone)]
pub struct FailoverAllocationValidator {
    pub rules: Vec<Rc<dyn FailoverAllocationRule>>,
}

impl FailoverAllocationValidator {
    pub fn validate(&self, item: &FailoverAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_allocation_validator() -> FailoverAllocationValidator {
    FailoverAllocationValidator {
        rules: vec![
            Rc::new(FailoverAllocationRegionLengthRule { max_length: 64 }) as Rc<dyn FailoverAllocationRule>,
            Rc::new(FailoverAllocationCurrencyLengthRule { max_length: 3600 }),
            Rc::new(FailoverAllocationRevisionRangeRule { min: 80, max: 86400 }),
            Rc::new(FailoverAllocationExpiresAtRangeRule { min: 64, max: 3600 }),
        ],
    }
}
