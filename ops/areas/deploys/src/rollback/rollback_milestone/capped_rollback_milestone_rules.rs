use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped rollback milestone must pass. Check returns its problems, or none.
pub trait CappedRollbackMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &CappedRollbackMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackMilestoneAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedRollbackMilestoneRule for CappedRollbackMilestoneAmountRangeRule {
    fn check(&self, item: &CappedRollbackMilestone) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackMilestoneRequiredNameRule {
    pub message: String,
}

impl CappedRollbackMilestoneRule for CappedRollbackMilestoneRequiredNameRule {
    fn check(&self, item: &CappedRollbackMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackMilestoneRequiredCodeRule {
    pub message: String,
}

impl CappedRollbackMilestoneRule for CappedRollbackMilestoneRequiredCodeRule {
    fn check(&self, item: &CappedRollbackMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped rollback milestone.
#[derive(Debug, Clone)]
pub struct CappedRollbackMilestoneValidator {
    pub rules: Vec<Rc<dyn CappedRollbackMilestoneRule>>,
}

impl CappedRollbackMilestoneValidator {
    pub fn validate(&self, item: &CappedRollbackMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedRollbackMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRollbackMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_rollback_milestone_validator() -> CappedRollbackMilestoneValidator {
    CappedRollbackMilestoneValidator {
        rules: vec![
            Rc::new(CappedRollbackMilestoneAmountRangeRule { min: 9.1, max: 2681.5 })
                as Rc<dyn CappedRollbackMilestoneRule>,
            Rc::new(CappedRollbackMilestoneRequiredNameRule {
                message: "capped rollback milestone needs a value".to_string(),
            }),
            Rc::new(CappedRollbackMilestoneRequiredCodeRule {
                message: "capped rollback milestone needs an entry".to_string(),
            }),
        ],
    }
}
