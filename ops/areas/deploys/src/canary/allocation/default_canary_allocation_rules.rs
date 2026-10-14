use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default canary allocation must pass. Check returns its problems, or none.
pub trait DefaultCanaryAllocationRule: std::fmt::Debug {
    fn check(&self, item: &DefaultCanaryAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCanaryAllocationTimezoneLengthRule {
    pub max_length: i64,
}

impl DefaultCanaryAllocationRule for DefaultCanaryAllocationTimezoneLengthRule {
    fn check(&self, item: &DefaultCanaryAllocation) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCanaryAllocationRequiredCodeRule {
    pub message: String,
}

impl DefaultCanaryAllocationRule for DefaultCanaryAllocationRequiredCodeRule {
    fn check(&self, item: &DefaultCanaryAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCanaryAllocationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultCanaryAllocationRule for DefaultCanaryAllocationMarginRangeRule {
    fn check(&self, item: &DefaultCanaryAllocation) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCanaryAllocationLabelLengthRule {
    pub max_length: i64,
}

impl DefaultCanaryAllocationRule for DefaultCanaryAllocationLabelLengthRule {
    fn check(&self, item: &DefaultCanaryAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default canary allocation.
#[derive(Debug, Clone)]
pub struct DefaultCanaryAllocationValidator {
    pub rules: Vec<Rc<dyn DefaultCanaryAllocationRule>>,
}

impl DefaultCanaryAllocationValidator {
    pub fn validate(&self, item: &DefaultCanaryAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultCanaryAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultCanaryAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_canary_allocation_validator() -> DefaultCanaryAllocationValidator {
    DefaultCanaryAllocationValidator {
        rules: vec![
            Rc::new(DefaultCanaryAllocationTimezoneLengthRule { max_length: 64 })
                as Rc<dyn DefaultCanaryAllocationRule>,
            Rc::new(DefaultCanaryAllocationRequiredCodeRule {
                message: "default canary allocation needs this field".to_string(),
            }),
            Rc::new(DefaultCanaryAllocationMarginRangeRule { min: 5.5, max: 3039.3 }),
            Rc::new(DefaultCanaryAllocationLabelLengthRule { max_length: 255 }),
        ],
    }
}
