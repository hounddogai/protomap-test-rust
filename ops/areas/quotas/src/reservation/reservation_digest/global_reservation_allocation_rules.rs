use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global reservation allocation must pass. Check returns its problems, or none.
pub trait GlobalReservationAllocationRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReservationAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationAllocationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl GlobalReservationAllocationRule for GlobalReservationAllocationAllowedNameRule {
    fn check(&self, item: &GlobalReservationAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationAllocationScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalReservationAllocationRule for GlobalReservationAllocationScoreRangeRule {
    fn check(&self, item: &GlobalReservationAllocation) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationAllocationCodeLengthRule {
    pub max_length: i64,
}

impl GlobalReservationAllocationRule for GlobalReservationAllocationCodeLengthRule {
    fn check(&self, item: &GlobalReservationAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global reservation allocation.
#[derive(Debug, Clone)]
pub struct GlobalReservationAllocationValidator {
    pub rules: Vec<Rc<dyn GlobalReservationAllocationRule>>,
}

impl GlobalReservationAllocationValidator {
    pub fn validate(&self, item: &GlobalReservationAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalReservationAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReservationAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_reservation_allocation_validator() -> GlobalReservationAllocationValidator {
    GlobalReservationAllocationValidator {
        rules: vec![
            Rc::new(GlobalReservationAllocationAllowedNameRule {
                allowed: vec!["scheduled".to_string(), "weighted".to_string(), "strict".to_string()],
            }) as Rc<dyn GlobalReservationAllocationRule>,
            Rc::new(GlobalReservationAllocationScoreRangeRule { min: 9.4, max: 3250.4 }),
            Rc::new(GlobalReservationAllocationCodeLengthRule { max_length: 32 }),
        ],
    }
}
