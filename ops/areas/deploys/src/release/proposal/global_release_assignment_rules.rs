use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global release assignment must pass. Check returns its problems, or none.
pub trait GlobalReleaseAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReleaseAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAssignmentActiveGuardRule {
    pub limit: i64,
}

impl GlobalReleaseAssignmentRule for GlobalReleaseAssignmentActiveGuardRule {
    fn check(&self, item: &GlobalReleaseAssignment) -> Vec<String> {
        if item.active && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while active", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAssignmentRequiredNameRule {
    pub message: String,
}

impl GlobalReleaseAssignmentRule for GlobalReleaseAssignmentRequiredNameRule {
    fn check(&self, item: &GlobalReleaseAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAssignmentRequiredCodeRule {
    pub message: String,
}

impl GlobalReleaseAssignmentRule for GlobalReleaseAssignmentRequiredCodeRule {
    fn check(&self, item: &GlobalReleaseAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAssignmentRequiredCurrencyRule {
    pub message: String,
}

impl GlobalReleaseAssignmentRule for GlobalReleaseAssignmentRequiredCurrencyRule {
    fn check(&self, item: &GlobalReleaseAssignment) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAssignmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalReleaseAssignmentRule for GlobalReleaseAssignmentCeilingRangeRule {
    fn check(&self, item: &GlobalReleaseAssignment) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global release assignment.
#[derive(Debug, Clone)]
pub struct GlobalReleaseAssignmentValidator {
    pub rules: Vec<Rc<dyn GlobalReleaseAssignmentRule>>,
}

impl GlobalReleaseAssignmentValidator {
    pub fn validate(&self, item: &GlobalReleaseAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalReleaseAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReleaseAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_release_assignment_validator() -> GlobalReleaseAssignmentValidator {
    GlobalReleaseAssignmentValidator {
        rules: vec![
            Rc::new(GlobalReleaseAssignmentActiveGuardRule { limit: 255 }) as Rc<dyn GlobalReleaseAssignmentRule>,
            Rc::new(GlobalReleaseAssignmentRequiredNameRule {
                message: "global release assignment needs an entry".to_string(),
            }),
            Rc::new(GlobalReleaseAssignmentRequiredCodeRule {
                message: "global release assignment needs a value".to_string(),
            }),
            Rc::new(GlobalReleaseAssignmentRequiredCurrencyRule {
                message: "global release assignment needs an entry".to_string(),
            }),
            Rc::new(GlobalReleaseAssignmentCeilingRangeRule { min: 0.9, max: 355.2 }),
        ],
    }
}
