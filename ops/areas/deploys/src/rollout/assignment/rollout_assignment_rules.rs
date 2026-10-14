use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout assignment must pass. Check returns its problems, or none.
pub trait RolloutAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RolloutAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAssignmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutAssignmentRule for RolloutAssignmentQuantityRangeRule {
    fn check(&self, item: &RolloutAssignment) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAssignmentSourceLengthRule {
    pub max_length: i64,
}

impl RolloutAssignmentRule for RolloutAssignmentSourceLengthRule {
    fn check(&self, item: &RolloutAssignment) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAssignmentRequiredCodeRule {
    pub message: String,
}

impl RolloutAssignmentRule for RolloutAssignmentRequiredCodeRule {
    fn check(&self, item: &RolloutAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout assignment.
#[derive(Debug, Clone)]
pub struct RolloutAssignmentValidator {
    pub rules: Vec<Rc<dyn RolloutAssignmentRule>>,
}

impl RolloutAssignmentValidator {
    pub fn validate(&self, item: &RolloutAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_assignment_validator() -> RolloutAssignmentValidator {
    RolloutAssignmentValidator {
        rules: vec![
            Rc::new(RolloutAssignmentQuantityRangeRule { min: 120, max: 120 }) as Rc<dyn RolloutAssignmentRule>,
            Rc::new(RolloutAssignmentSourceLengthRule { max_length: 120 }),
            Rc::new(RolloutAssignmentRequiredCodeRule {
                message: "rollout assignment needs an entry".to_string(),
            }),
        ],
    }
}
