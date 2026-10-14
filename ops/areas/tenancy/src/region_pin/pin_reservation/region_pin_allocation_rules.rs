use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin allocation must pass. Check returns its problems, or none.
pub trait RegionPinAllocationRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAllocationRequiredChecksumRule {
    pub message: String,
}

impl RegionPinAllocationRule for RegionPinAllocationRequiredChecksumRule {
    fn check(&self, item: &RegionPinAllocation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAllocationCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinAllocationRule for RegionPinAllocationCodeLengthRule {
    fn check(&self, item: &RegionPinAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAllocationCategoryLengthRule {
    pub max_length: i64,
}

impl RegionPinAllocationRule for RegionPinAllocationCategoryLengthRule {
    fn check(&self, item: &RegionPinAllocation) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAllocationRequiredTitleRule {
    pub message: String,
}

impl RegionPinAllocationRule for RegionPinAllocationRequiredTitleRule {
    fn check(&self, item: &RegionPinAllocation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAllocationRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinAllocationRule for RegionPinAllocationRotationDaysRangeRule {
    fn check(&self, item: &RegionPinAllocation) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAllocationCodePrefixRule {
    pub prefix: String,
}

impl RegionPinAllocationRule for RegionPinAllocationCodePrefixRule {
    fn check(&self, item: &RegionPinAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin allocation.
#[derive(Debug, Clone)]
pub struct RegionPinAllocationValidator {
    pub rules: Vec<Rc<dyn RegionPinAllocationRule>>,
}

impl RegionPinAllocationValidator {
    pub fn validate(&self, item: &RegionPinAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_allocation_validator() -> RegionPinAllocationValidator {
    RegionPinAllocationValidator {
        rules: vec![
            Rc::new(RegionPinAllocationRequiredChecksumRule {
                message: "region pin allocation needs an entry".to_string(),
            }) as Rc<dyn RegionPinAllocationRule>,
            Rc::new(RegionPinAllocationCodeLengthRule { max_length: 86400 }),
            Rc::new(RegionPinAllocationCategoryLengthRule { max_length: 1000 }),
            Rc::new(RegionPinAllocationRequiredTitleRule {
                message: "region pin allocation needs this field".to_string(),
            }),
            Rc::new(RegionPinAllocationRotationDaysRangeRule { min: 80, max: 120 }),
            Rc::new(RegionPinAllocationCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
