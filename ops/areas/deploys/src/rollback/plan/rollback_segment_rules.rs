use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback segment must pass. Check returns its problems, or none.
pub trait RollbackSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RollbackSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSegmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollbackSegmentRule for RollbackSegmentAllowedNameRule {
    fn check(&self, item: &RollbackSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSegmentBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackSegmentRule for RollbackSegmentBudgetRangeRule {
    fn check(&self, item: &RollbackSegment) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSegmentRequiredCodeRule {
    pub message: String,
}

impl RollbackSegmentRule for RollbackSegmentRequiredCodeRule {
    fn check(&self, item: &RollbackSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSegmentRequiredNameRule {
    pub message: String,
}

impl RollbackSegmentRule for RollbackSegmentRequiredNameRule {
    fn check(&self, item: &RollbackSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSegmentCodeLengthRule {
    pub max_length: i64,
}

impl RollbackSegmentRule for RollbackSegmentCodeLengthRule {
    fn check(&self, item: &RollbackSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback segment.
#[derive(Debug, Clone)]
pub struct RollbackSegmentValidator {
    pub rules: Vec<Rc<dyn RollbackSegmentRule>>,
}

impl RollbackSegmentValidator {
    pub fn validate(&self, item: &RollbackSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_segment_validator() -> RollbackSegmentValidator {
    RollbackSegmentValidator {
        rules: vec![
            Rc::new(RollbackSegmentAllowedNameRule {
                allowed: vec!["fallback".to_string(), "default".to_string()],
            }) as Rc<dyn RollbackSegmentRule>,
            Rc::new(RollbackSegmentBudgetRangeRule { min: 3.9, max: 1599.6 }),
            Rc::new(RollbackSegmentRequiredCodeRule {
                message: "rollback segment needs an entry".to_string(),
            }),
            Rc::new(RollbackSegmentRequiredNameRule {
                message: "rollback segment needs an entry".to_string(),
            }),
            Rc::new(RollbackSegmentCodeLengthRule { max_length: 3600 }),
        ],
    }
}
