use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default federation allocation must pass. Check returns its problems, or none.
pub trait DefaultFederationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFederationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocationAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl DefaultFederationAllocationRule for DefaultFederationAllocationAllowedSegmentRule {
    fn check(&self, item: &DefaultFederationAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is missing", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocationRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultFederationAllocationRule for DefaultFederationAllocationRevisionRangeRule {
    fn check(&self, item: &DefaultFederationAllocation) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocationSegmentLengthRule {
    pub max_length: i64,
}

impl DefaultFederationAllocationRule for DefaultFederationAllocationSegmentLengthRule {
    fn check(&self, item: &DefaultFederationAllocation) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocationSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultFederationAllocationRule for DefaultFederationAllocationSequenceRangeRule {
    fn check(&self, item: &DefaultFederationAllocation) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocationDescriptionLengthRule {
    pub max_length: i64,
}

impl DefaultFederationAllocationRule for DefaultFederationAllocationDescriptionLengthRule {
    fn check(&self, item: &DefaultFederationAllocation) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a default federation allocation.
#[derive(Debug, Clone)]
pub struct DefaultFederationAllocationValidator {
    pub rules: Vec<Rc<dyn DefaultFederationAllocationRule>>,
}

impl DefaultFederationAllocationValidator {
    pub fn validate(&self, item: &DefaultFederationAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultFederationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFederationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_federation_allocation_validator() -> DefaultFederationAllocationValidator {
    DefaultFederationAllocationValidator {
        rules: vec![
            Rc::new(DefaultFederationAllocationAllowedSegmentRule { allowed: Vec::new() })
                as Rc<dyn DefaultFederationAllocationRule>,
            Rc::new(DefaultFederationAllocationRevisionRangeRule { min: 86400, max: 86400 }),
            Rc::new(DefaultFederationAllocationSegmentLengthRule { max_length: 16 }),
            Rc::new(DefaultFederationAllocationSequenceRangeRule { min: 120, max: 120 }),
            Rc::new(DefaultFederationAllocationDescriptionLengthRule { max_length: 80 }),
        ],
    }
}
