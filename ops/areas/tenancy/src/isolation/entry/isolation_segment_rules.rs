use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation segment must pass. Check returns its problems, or none.
pub trait IsolationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &IsolationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSegmentRequiredCodeRule {
    pub message: String,
}

impl IsolationSegmentRule for IsolationSegmentRequiredCodeRule {
    fn check(&self, item: &IsolationSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSegmentRequiredTitleRule {
    pub message: String,
}

impl IsolationSegmentRule for IsolationSegmentRequiredTitleRule {
    fn check(&self, item: &IsolationSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSegmentCodeLengthRule {
    pub max_length: i64,
}

impl IsolationSegmentRule for IsolationSegmentCodeLengthRule {
    fn check(&self, item: &IsolationSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSegmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl IsolationSegmentRule for IsolationSegmentAllowedCodeRule {
    fn check(&self, item: &IsolationSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSegmentEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationSegmentRule for IsolationSegmentEndsAtRangeRule {
    fn check(&self, item: &IsolationSegment) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSegmentTitleLengthRule {
    pub max_length: i64,
}

impl IsolationSegmentRule for IsolationSegmentTitleLengthRule {
    fn check(&self, item: &IsolationSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation segment.
#[derive(Debug, Clone)]
pub struct IsolationSegmentValidator {
    pub rules: Vec<Rc<dyn IsolationSegmentRule>>,
}

impl IsolationSegmentValidator {
    pub fn validate(&self, item: &IsolationSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_segment_validator() -> IsolationSegmentValidator {
    IsolationSegmentValidator {
        rules: vec![
            Rc::new(IsolationSegmentRequiredCodeRule {
                message: "isolation segment needs an entry".to_string(),
            }) as Rc<dyn IsolationSegmentRule>,
            Rc::new(IsolationSegmentRequiredTitleRule {
                message: "isolation segment needs an entry".to_string(),
            }),
            Rc::new(IsolationSegmentCodeLengthRule { max_length: 3600 }),
            Rc::new(IsolationSegmentAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "strict".to_string(), "weighted".to_string()],
            }),
            Rc::new(IsolationSegmentEndsAtRangeRule { min: 255, max: 86400 }),
            Rc::new(IsolationSegmentTitleLengthRule { max_length: 80 }),
        ],
    }
}
