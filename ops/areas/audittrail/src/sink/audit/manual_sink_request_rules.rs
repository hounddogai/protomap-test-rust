use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual sink request must pass. Check returns its problems, or none.
pub trait ManualSinkRequestRule: std::fmt::Debug {
    fn check(&self, item: &ManualSinkRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkRequestAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl ManualSinkRequestRule for ManualSinkRequestAllowedDescriptionRule {
    fn check(&self, item: &ManualSinkRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is unknown", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkRequestRequiredTitleRule {
    pub message: String,
}

impl ManualSinkRequestRule for ManualSinkRequestRequiredTitleRule {
    fn check(&self, item: &ManualSinkRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkRequestSegmentLengthRule {
    pub max_length: i64,
}

impl ManualSinkRequestRule for ManualSinkRequestSegmentLengthRule {
    fn check(&self, item: &ManualSinkRequest) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkRequestRequiredCodeRule {
    pub message: String,
}

impl ManualSinkRequestRule for ManualSinkRequestRequiredCodeRule {
    fn check(&self, item: &ManualSinkRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual sink request.
#[derive(Debug, Clone)]
pub struct ManualSinkRequestValidator {
    pub rules: Vec<Rc<dyn ManualSinkRequestRule>>,
}

impl ManualSinkRequestValidator {
    pub fn validate(&self, item: &ManualSinkRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualSinkRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSinkRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_sink_request_validator() -> ManualSinkRequestValidator {
    ManualSinkRequestValidator {
        rules: vec![
            Rc::new(ManualSinkRequestAllowedDescriptionRule { allowed: Vec::new() }) as Rc<dyn ManualSinkRequestRule>,
            Rc::new(ManualSinkRequestRequiredTitleRule {
                message: "manual sink request needs an entry".to_string(),
            }),
            Rc::new(ManualSinkRequestSegmentLengthRule { max_length: 120 }),
            Rc::new(ManualSinkRequestRequiredCodeRule {
                message: "manual sink request needs this field".to_string(),
            }),
        ],
    }
}
