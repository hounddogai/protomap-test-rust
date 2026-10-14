use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled binding allocation must pass. Check returns its problems, or none.
pub trait ScheduledBindingAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledBindingAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocationLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledBindingAllocationRule for ScheduledBindingAllocationLabelLengthRule {
    fn check(&self, item: &ScheduledBindingAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocationQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledBindingAllocationRule for ScheduledBindingAllocationQuantityRangeRule {
    fn check(&self, item: &ScheduledBindingAllocation) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocationApprovedGuardRule {
    pub limit: i64,
}

impl ScheduledBindingAllocationRule for ScheduledBindingAllocationApprovedGuardRule {
    fn check(&self, item: &ScheduledBindingAllocation) -> Vec<String> {
        if item.approved && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while approved",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocationCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledBindingAllocationRule for ScheduledBindingAllocationCodeLengthRule {
    fn check(&self, item: &ScheduledBindingAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocationPhoneLengthRule {
    pub max_length: i64,
}

impl ScheduledBindingAllocationRule for ScheduledBindingAllocationPhoneLengthRule {
    fn check(&self, item: &ScheduledBindingAllocation) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled binding allocation.
#[derive(Debug, Clone)]
pub struct ScheduledBindingAllocationValidator {
    pub rules: Vec<Rc<dyn ScheduledBindingAllocationRule>>,
}

impl ScheduledBindingAllocationValidator {
    pub fn validate(&self, item: &ScheduledBindingAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledBindingAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledBindingAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_binding_allocation_validator() -> ScheduledBindingAllocationValidator {
    ScheduledBindingAllocationValidator {
        rules: vec![
            Rc::new(ScheduledBindingAllocationLabelLengthRule { max_length: 86400 })
                as Rc<dyn ScheduledBindingAllocationRule>,
            Rc::new(ScheduledBindingAllocationQuantityRangeRule { min: 64, max: 64 }),
            Rc::new(ScheduledBindingAllocationApprovedGuardRule { limit: 120 }),
            Rc::new(ScheduledBindingAllocationCodeLengthRule { max_length: 16 }),
            Rc::new(ScheduledBindingAllocationPhoneLengthRule { max_length: 80 }),
        ],
    }
}
