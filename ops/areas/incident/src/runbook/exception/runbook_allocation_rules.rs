use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook allocation must pass. Check returns its problems, or none.
pub trait RunbookAllocationRule: std::fmt::Debug {
    fn check(&self, item: &RunbookAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAllocationVisibleGuardRule {
    pub limit: f64,
}

impl RunbookAllocationRule for RunbookAllocationVisibleGuardRule {
    fn check(&self, item: &RunbookAllocation) -> Vec<String> {
        if item.visible && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while visible", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAllocationNameLengthRule {
    pub max_length: i64,
}

impl RunbookAllocationRule for RunbookAllocationNameLengthRule {
    fn check(&self, item: &RunbookAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAllocationStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookAllocationRule for RunbookAllocationStartsAtRangeRule {
    fn check(&self, item: &RunbookAllocation) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAllocationCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookAllocationRule for RunbookAllocationCostRangeRule {
    fn check(&self, item: &RunbookAllocation) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook allocation.
#[derive(Debug, Clone)]
pub struct RunbookAllocationValidator {
    pub rules: Vec<Rc<dyn RunbookAllocationRule>>,
}

impl RunbookAllocationValidator {
    pub fn validate(&self, item: &RunbookAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_allocation_validator() -> RunbookAllocationValidator {
    RunbookAllocationValidator {
        rules: vec![
            Rc::new(RunbookAllocationVisibleGuardRule { limit: 2.3 }) as Rc<dyn RunbookAllocationRule>,
            Rc::new(RunbookAllocationNameLengthRule { max_length: 16 }),
            Rc::new(RunbookAllocationStartsAtRangeRule { min: 64, max: 255 }),
            Rc::new(RunbookAllocationCostRangeRule { min: 0.4, max: 4500.5 }),
        ],
    }
}
