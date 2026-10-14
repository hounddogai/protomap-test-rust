use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched audit event budget must pass. Check returns its problems, or none.
pub trait BatchedAuditEventBudgetRule: std::fmt::Debug {
    fn check(&self, item: &BatchedAuditEventBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventBudgetThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedAuditEventBudgetRule for BatchedAuditEventBudgetThresholdRangeRule {
    fn check(&self, item: &BatchedAuditEventBudget) -> Vec<String> {
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
pub struct BatchedAuditEventBudgetTitleLengthRule {
    pub max_length: i64,
}

impl BatchedAuditEventBudgetRule for BatchedAuditEventBudgetTitleLengthRule {
    fn check(&self, item: &BatchedAuditEventBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventBudgetLabelLengthRule {
    pub max_length: i64,
}

impl BatchedAuditEventBudgetRule for BatchedAuditEventBudgetLabelLengthRule {
    fn check(&self, item: &BatchedAuditEventBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventBudgetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedAuditEventBudgetRule for BatchedAuditEventBudgetMarginRangeRule {
    fn check(&self, item: &BatchedAuditEventBudget) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched audit event budget.
#[derive(Debug, Clone)]
pub struct BatchedAuditEventBudgetValidator {
    pub rules: Vec<Rc<dyn BatchedAuditEventBudgetRule>>,
}

impl BatchedAuditEventBudgetValidator {
    pub fn validate(&self, item: &BatchedAuditEventBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedAuditEventBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedAuditEventBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_audit_event_budget_validator() -> BatchedAuditEventBudgetValidator {
    BatchedAuditEventBudgetValidator {
        rules: vec![
            Rc::new(BatchedAuditEventBudgetThresholdRangeRule { min: 9.9, max: 3463.0 })
                as Rc<dyn BatchedAuditEventBudgetRule>,
            Rc::new(BatchedAuditEventBudgetTitleLengthRule { max_length: 255 }),
            Rc::new(BatchedAuditEventBudgetLabelLengthRule { max_length: 120 }),
            Rc::new(BatchedAuditEventBudgetMarginRangeRule { min: 7.0, max: 781.8 }),
        ],
    }
}
