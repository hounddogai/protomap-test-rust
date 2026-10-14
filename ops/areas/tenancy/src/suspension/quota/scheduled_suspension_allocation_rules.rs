use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled suspension allocation must pass. Check returns its problems, or none.
pub trait ScheduledSuspensionAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSuspensionAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionAllocationRequiredPhoneRule {
    pub message: String,
}

impl ScheduledSuspensionAllocationRule for ScheduledSuspensionAllocationRequiredPhoneRule {
    fn check(&self, item: &ScheduledSuspensionAllocation) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionAllocationRequiredNameRule {
    pub message: String,
}

impl ScheduledSuspensionAllocationRule for ScheduledSuspensionAllocationRequiredNameRule {
    fn check(&self, item: &ScheduledSuspensionAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionAllocationFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledSuspensionAllocationRule for ScheduledSuspensionAllocationFloorValueRangeRule {
    fn check(&self, item: &ScheduledSuspensionAllocation) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionAllocationAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl ScheduledSuspensionAllocationRule for ScheduledSuspensionAllocationAllowedPhoneRule {
    fn check(&self, item: &ScheduledSuspensionAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is duplicated", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionAllocationCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionAllocationRule for ScheduledSuspensionAllocationCodeLengthRule {
    fn check(&self, item: &ScheduledSuspensionAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled suspension allocation.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionAllocationValidator {
    pub rules: Vec<Rc<dyn ScheduledSuspensionAllocationRule>>,
}

impl ScheduledSuspensionAllocationValidator {
    pub fn validate(&self, item: &ScheduledSuspensionAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSuspensionAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSuspensionAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_suspension_allocation_validator() -> ScheduledSuspensionAllocationValidator {
    ScheduledSuspensionAllocationValidator {
        rules: vec![
            Rc::new(ScheduledSuspensionAllocationRequiredPhoneRule {
                message: "scheduled suspension allocation needs a value".to_string(),
            }) as Rc<dyn ScheduledSuspensionAllocationRule>,
            Rc::new(ScheduledSuspensionAllocationRequiredNameRule {
                message: "scheduled suspension allocation needs an entry".to_string(),
            }),
            Rc::new(ScheduledSuspensionAllocationFloorValueRangeRule { min: 8.8, max: 1512.5 }),
            Rc::new(ScheduledSuspensionAllocationAllowedPhoneRule {
                allowed: vec!["adaptive".to_string(), "manual".to_string(), "manual".to_string()],
            }),
            Rc::new(ScheduledSuspensionAllocationCodeLengthRule { max_length: 3600 }),
        ],
    }
}
