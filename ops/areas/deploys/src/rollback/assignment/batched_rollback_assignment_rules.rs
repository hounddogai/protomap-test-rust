use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched rollback assignment must pass. Check returns its problems, or none.
pub trait BatchedRollbackAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRollbackAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackAssignmentCodePrefixRule {
    pub prefix: String,
}

impl BatchedRollbackAssignmentRule for BatchedRollbackAssignmentCodePrefixRule {
    fn check(&self, item: &BatchedRollbackAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackAssignmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedRollbackAssignmentRule for BatchedRollbackAssignmentAmountRangeRule {
    fn check(&self, item: &BatchedRollbackAssignment) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackAssignmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRollbackAssignmentRule for BatchedRollbackAssignmentQuantityRangeRule {
    fn check(&self, item: &BatchedRollbackAssignment) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched rollback assignment.
#[derive(Debug, Clone)]
pub struct BatchedRollbackAssignmentValidator {
    pub rules: Vec<Rc<dyn BatchedRollbackAssignmentRule>>,
}

impl BatchedRollbackAssignmentValidator {
    pub fn validate(&self, item: &BatchedRollbackAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRollbackAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRollbackAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_rollback_assignment_validator() -> BatchedRollbackAssignmentValidator {
    BatchedRollbackAssignmentValidator {
        rules: vec![
            Rc::new(BatchedRollbackAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn BatchedRollbackAssignmentRule>,
            Rc::new(BatchedRollbackAssignmentAmountRangeRule { min: 2.1, max: 2087.3 }),
            Rc::new(BatchedRollbackAssignmentQuantityRangeRule { min: 16, max: 86400 }),
        ],
    }
}
