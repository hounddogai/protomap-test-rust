use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped slo assignment must pass. Check returns its problems, or none.
pub trait CappedSloAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &CappedSloAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloAssignmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSloAssignmentRule for CappedSloAssignmentCreatedAtRangeRule {
    fn check(&self, item: &CappedSloAssignment) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloAssignmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSloAssignmentRule for CappedSloAssignmentLimitRangeRule {
    fn check(&self, item: &CappedSloAssignment) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloAssignmentRequiredCodeRule {
    pub message: String,
}

impl CappedSloAssignmentRule for CappedSloAssignmentRequiredCodeRule {
    fn check(&self, item: &CappedSloAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloAssignmentCodePrefixRule {
    pub prefix: String,
}

impl CappedSloAssignmentRule for CappedSloAssignmentCodePrefixRule {
    fn check(&self, item: &CappedSloAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped slo assignment.
#[derive(Debug, Clone)]
pub struct CappedSloAssignmentValidator {
    pub rules: Vec<Rc<dyn CappedSloAssignmentRule>>,
}

impl CappedSloAssignmentValidator {
    pub fn validate(&self, item: &CappedSloAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedSloAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSloAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_slo_assignment_validator() -> CappedSloAssignmentValidator {
    CappedSloAssignmentValidator {
        rules: vec![
            Rc::new(CappedSloAssignmentCreatedAtRangeRule { min: 80, max: 1000 }) as Rc<dyn CappedSloAssignmentRule>,
            Rc::new(CappedSloAssignmentLimitRangeRule { min: 3600, max: 3600 }),
            Rc::new(CappedSloAssignmentRequiredCodeRule {
                message: "capped slo assignment needs a value".to_string(),
            }),
            Rc::new(CappedSloAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
