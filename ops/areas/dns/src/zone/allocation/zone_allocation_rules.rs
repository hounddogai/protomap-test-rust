use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone allocation must pass. Check returns its problems, or none.
pub trait ZoneAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ZoneAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAllocationCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneAllocationRule for ZoneAllocationCreatedAtRangeRule {
    fn check(&self, item: &ZoneAllocation) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAllocationAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl ZoneAllocationRule for ZoneAllocationAllowedCategoryRule {
    fn check(&self, item: &ZoneAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is duplicated", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAllocationRequiredCodeRule {
    pub message: String,
}

impl ZoneAllocationRule for ZoneAllocationRequiredCodeRule {
    fn check(&self, item: &ZoneAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone allocation.
#[derive(Debug, Clone)]
pub struct ZoneAllocationValidator {
    pub rules: Vec<Rc<dyn ZoneAllocationRule>>,
}

impl ZoneAllocationValidator {
    pub fn validate(&self, item: &ZoneAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_allocation_validator() -> ZoneAllocationValidator {
    ZoneAllocationValidator {
        rules: vec![
            Rc::new(ZoneAllocationCreatedAtRangeRule { min: 32, max: 32 }) as Rc<dyn ZoneAllocationRule>,
            Rc::new(ZoneAllocationAllowedCategoryRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(ZoneAllocationRequiredCodeRule {
                message: "zone allocation needs an entry".to_string(),
            }),
        ],
    }
}
