use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast segment must pass. Check returns its problems, or none.
pub trait ForecastSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ForecastSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSegmentCodeLengthRule {
    pub max_length: i64,
}

impl ForecastSegmentRule for ForecastSegmentCodeLengthRule {
    fn check(&self, item: &ForecastSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSegmentRequiredPhoneRule {
    pub message: String,
}

impl ForecastSegmentRule for ForecastSegmentRequiredPhoneRule {
    fn check(&self, item: &ForecastSegment) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSegmentRequiredCodeRule {
    pub message: String,
}

impl ForecastSegmentRule for ForecastSegmentRequiredCodeRule {
    fn check(&self, item: &ForecastSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSegmentRequiredNotesRule {
    pub message: String,
}

impl ForecastSegmentRule for ForecastSegmentRequiredNotesRule {
    fn check(&self, item: &ForecastSegment) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSegmentAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl ForecastSegmentRule for ForecastSegmentAllowedNotesRule {
    fn check(&self, item: &ForecastSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is out of range", item.notes)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast segment.
#[derive(Debug, Clone)]
pub struct ForecastSegmentValidator {
    pub rules: Vec<Rc<dyn ForecastSegmentRule>>,
}

impl ForecastSegmentValidator {
    pub fn validate(&self, item: &ForecastSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_segment_validator() -> ForecastSegmentValidator {
    ForecastSegmentValidator {
        rules: vec![
            Rc::new(ForecastSegmentCodeLengthRule { max_length: 16 }) as Rc<dyn ForecastSegmentRule>,
            Rc::new(ForecastSegmentRequiredPhoneRule {
                message: "forecast segment needs a value".to_string(),
            }),
            Rc::new(ForecastSegmentRequiredCodeRule {
                message: "forecast segment needs this field".to_string(),
            }),
            Rc::new(ForecastSegmentRequiredNotesRule {
                message: "forecast segment needs this field".to_string(),
            }),
            Rc::new(ForecastSegmentAllowedNotesRule {
                allowed: vec!["primary".to_string()],
            }),
        ],
    }
}
