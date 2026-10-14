use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global health check allocation must pass. Check returns its problems, or none.
pub trait GlobalHealthCheckAllocationRule: std::fmt::Debug {
    fn check(&self, item: &GlobalHealthCheckAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckAllocationCodeLengthRule {
    pub max_length: i64,
}

impl GlobalHealthCheckAllocationRule for GlobalHealthCheckAllocationCodeLengthRule {
    fn check(&self, item: &GlobalHealthCheckAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckAllocationCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalHealthCheckAllocationRule for GlobalHealthCheckAllocationCapacityRangeRule {
    fn check(&self, item: &GlobalHealthCheckAllocation) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckAllocationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalHealthCheckAllocationRule for GlobalHealthCheckAllocationAllowedCodeRule {
    fn check(&self, item: &GlobalHealthCheckAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckAllocationRequiredTitleRule {
    pub message: String,
}

impl GlobalHealthCheckAllocationRule for GlobalHealthCheckAllocationRequiredTitleRule {
    fn check(&self, item: &GlobalHealthCheckAllocation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global health check allocation.
#[derive(Debug, Clone)]
pub struct GlobalHealthCheckAllocationValidator {
    pub rules: Vec<Rc<dyn GlobalHealthCheckAllocationRule>>,
}

impl GlobalHealthCheckAllocationValidator {
    pub fn validate(&self, item: &GlobalHealthCheckAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalHealthCheckAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalHealthCheckAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_health_check_allocation_validator() -> GlobalHealthCheckAllocationValidator {
    GlobalHealthCheckAllocationValidator {
        rules: vec![
            Rc::new(GlobalHealthCheckAllocationCodeLengthRule { max_length: 80 })
                as Rc<dyn GlobalHealthCheckAllocationRule>,
            Rc::new(GlobalHealthCheckAllocationCapacityRangeRule { min: 64, max: 64 }),
            Rc::new(GlobalHealthCheckAllocationAllowedCodeRule {
                allowed: vec!["tiered".to_string()],
            }),
            Rc::new(GlobalHealthCheckAllocationRequiredTitleRule {
                message: "global health check allocation needs this field".to_string(),
            }),
        ],
    }
}
