use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional timeline cycle must pass. Check returns its problems, or none.
pub trait RegionalTimelineCycleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalTimelineCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineCycleCodePrefixRule {
    pub prefix: String,
}

impl RegionalTimelineCycleRule for RegionalTimelineCycleCodePrefixRule {
    fn check(&self, item: &RegionalTimelineCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineCycleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalTimelineCycleRule for RegionalTimelineCycleRevisionRangeRule {
    fn check(&self, item: &RegionalTimelineCycle) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineCycleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RegionalTimelineCycleRule for RegionalTimelineCycleAllowedTitleRule {
    fn check(&self, item: &RegionalTimelineCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineCycleTitleLengthRule {
    pub max_length: i64,
}

impl RegionalTimelineCycleRule for RegionalTimelineCycleTitleLengthRule {
    fn check(&self, item: &RegionalTimelineCycle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional timeline cycle.
#[derive(Debug, Clone)]
pub struct RegionalTimelineCycleValidator {
    pub rules: Vec<Rc<dyn RegionalTimelineCycleRule>>,
}

impl RegionalTimelineCycleValidator {
    pub fn validate(&self, item: &RegionalTimelineCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalTimelineCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalTimelineCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_timeline_cycle_validator() -> RegionalTimelineCycleValidator {
    RegionalTimelineCycleValidator {
        rules: vec![
            Rc::new(RegionalTimelineCycleCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RegionalTimelineCycleRule>,
            Rc::new(RegionalTimelineCycleRevisionRangeRule { min: 16, max: 120 }),
            Rc::new(RegionalTimelineCycleAllowedTitleRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(RegionalTimelineCycleTitleLengthRule { max_length: 16 }),
        ],
    }
}
