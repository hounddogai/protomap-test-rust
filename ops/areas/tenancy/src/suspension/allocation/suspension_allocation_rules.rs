use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension allocation must pass. Check returns its problems, or none.
pub trait SuspensionAllocationRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAllocationPrincipalLengthRule {
    pub max_length: i64,
}

impl SuspensionAllocationRule for SuspensionAllocationPrincipalLengthRule {
    fn check(&self, item: &SuspensionAllocation) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAllocationRequiredReviewerRule {
    pub message: String,
}

impl SuspensionAllocationRule for SuspensionAllocationRequiredReviewerRule {
    fn check(&self, item: &SuspensionAllocation) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAllocationAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionAllocationRule for SuspensionAllocationAmountRangeRule {
    fn check(&self, item: &SuspensionAllocation) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension allocation.
#[derive(Debug, Clone)]
pub struct SuspensionAllocationValidator {
    pub rules: Vec<Rc<dyn SuspensionAllocationRule>>,
}

impl SuspensionAllocationValidator {
    pub fn validate(&self, item: &SuspensionAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_allocation_validator() -> SuspensionAllocationValidator {
    SuspensionAllocationValidator {
        rules: vec![
            Rc::new(SuspensionAllocationPrincipalLengthRule { max_length: 255 }) as Rc<dyn SuspensionAllocationRule>,
            Rc::new(SuspensionAllocationRequiredReviewerRule {
                message: "suspension allocation needs an entry".to_string(),
            }),
            Rc::new(SuspensionAllocationAmountRangeRule { min: 3.2, max: 3771.7 }),
        ],
    }
}
