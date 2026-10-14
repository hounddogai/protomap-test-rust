use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline allocation must pass. Check returns its problems, or none.
pub trait TimelineAllocationRule: std::fmt::Debug {
    fn check(&self, item: &TimelineAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAllocationCountryLengthRule {
    pub max_length: i64,
}

impl TimelineAllocationRule for TimelineAllocationCountryLengthRule {
    fn check(&self, item: &TimelineAllocation) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAllocationRequiredNameRule {
    pub message: String,
}

impl TimelineAllocationRule for TimelineAllocationRequiredNameRule {
    fn check(&self, item: &TimelineAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAllocationPhoneLengthRule {
    pub max_length: i64,
}

impl TimelineAllocationRule for TimelineAllocationPhoneLengthRule {
    fn check(&self, item: &TimelineAllocation) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAllocationCodeLengthRule {
    pub max_length: i64,
}

impl TimelineAllocationRule for TimelineAllocationCodeLengthRule {
    fn check(&self, item: &TimelineAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAllocationAllowedNameRule {
    pub allowed: Vec<String>,
}

impl TimelineAllocationRule for TimelineAllocationAllowedNameRule {
    fn check(&self, item: &TimelineAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline allocation.
#[derive(Debug, Clone)]
pub struct TimelineAllocationValidator {
    pub rules: Vec<Rc<dyn TimelineAllocationRule>>,
}

impl TimelineAllocationValidator {
    pub fn validate(&self, item: &TimelineAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_allocation_validator() -> TimelineAllocationValidator {
    TimelineAllocationValidator {
        rules: vec![
            Rc::new(TimelineAllocationCountryLengthRule { max_length: 64 }) as Rc<dyn TimelineAllocationRule>,
            Rc::new(TimelineAllocationRequiredNameRule {
                message: "timeline allocation needs this field".to_string(),
            }),
            Rc::new(TimelineAllocationPhoneLengthRule { max_length: 120 }),
            Rc::new(TimelineAllocationCodeLengthRule { max_length: 3600 }),
            Rc::new(TimelineAllocationAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}
