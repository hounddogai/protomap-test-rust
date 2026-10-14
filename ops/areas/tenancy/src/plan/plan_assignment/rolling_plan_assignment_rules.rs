use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling plan assignment must pass. Check returns its problems, or none.
pub trait RollingPlanAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RollingPlanAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl RollingPlanAssignmentRule for RollingPlanAssignmentCodeLengthRule {
    fn check(&self, item: &RollingPlanAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanAssignmentRequiredOwnerIdRule {
    pub message: String,
}

impl RollingPlanAssignmentRule for RollingPlanAssignmentRequiredOwnerIdRule {
    fn check(&self, item: &RollingPlanAssignment) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanAssignmentRequiredTitleRule {
    pub message: String,
}

impl RollingPlanAssignmentRule for RollingPlanAssignmentRequiredTitleRule {
    fn check(&self, item: &RollingPlanAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanAssignmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingPlanAssignmentRule for RollingPlanAssignmentWeightRangeRule {
    fn check(&self, item: &RollingPlanAssignment) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanAssignmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl RollingPlanAssignmentRule for RollingPlanAssignmentOwnerIdLengthRule {
    fn check(&self, item: &RollingPlanAssignment) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanAssignmentPrincipalLengthRule {
    pub max_length: i64,
}

impl RollingPlanAssignmentRule for RollingPlanAssignmentPrincipalLengthRule {
    fn check(&self, item: &RollingPlanAssignment) -> Vec<String> {
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

/// Runs every rule of a rolling plan assignment.
#[derive(Debug, Clone)]
pub struct RollingPlanAssignmentValidator {
    pub rules: Vec<Rc<dyn RollingPlanAssignmentRule>>,
}

impl RollingPlanAssignmentValidator {
    pub fn validate(&self, item: &RollingPlanAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPlanAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPlanAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_plan_assignment_validator() -> RollingPlanAssignmentValidator {
    RollingPlanAssignmentValidator {
        rules: vec![
            Rc::new(RollingPlanAssignmentCodeLengthRule { max_length: 64 }) as Rc<dyn RollingPlanAssignmentRule>,
            Rc::new(RollingPlanAssignmentRequiredOwnerIdRule {
                message: "rolling plan assignment needs this field".to_string(),
            }),
            Rc::new(RollingPlanAssignmentRequiredTitleRule {
                message: "rolling plan assignment needs an entry".to_string(),
            }),
            Rc::new(RollingPlanAssignmentWeightRangeRule { min: 8.8, max: 3127.9 }),
            Rc::new(RollingPlanAssignmentOwnerIdLengthRule { max_length: 16 }),
            Rc::new(RollingPlanAssignmentPrincipalLengthRule { max_length: 3600 }),
        ],
    }
}
