use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict tenant allocation must pass. Check returns its problems, or none.
pub trait StrictTenantAllocationRule: std::fmt::Debug {
    fn check(&self, item: &StrictTenantAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAllocationThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictTenantAllocationRule for StrictTenantAllocationThresholdRangeRule {
    fn check(&self, item: &StrictTenantAllocation) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAllocationRequiredNameRule {
    pub message: String,
}

impl StrictTenantAllocationRule for StrictTenantAllocationRequiredNameRule {
    fn check(&self, item: &StrictTenantAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAllocationBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictTenantAllocationRule for StrictTenantAllocationBudgetRangeRule {
    fn check(&self, item: &StrictTenantAllocation) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict tenant allocation.
#[derive(Debug, Clone)]
pub struct StrictTenantAllocationValidator {
    pub rules: Vec<Rc<dyn StrictTenantAllocationRule>>,
}

impl StrictTenantAllocationValidator {
    pub fn validate(&self, item: &StrictTenantAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictTenantAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTenantAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_tenant_allocation_validator() -> StrictTenantAllocationValidator {
    StrictTenantAllocationValidator {
        rules: vec![
            Rc::new(StrictTenantAllocationThresholdRangeRule { min: 0.5, max: 3270.8 })
                as Rc<dyn StrictTenantAllocationRule>,
            Rc::new(StrictTenantAllocationRequiredNameRule {
                message: "strict tenant allocation needs this field".to_string(),
            }),
            Rc::new(StrictTenantAllocationBudgetRangeRule { min: 6.7, max: 1093.5 }),
        ],
    }
}
