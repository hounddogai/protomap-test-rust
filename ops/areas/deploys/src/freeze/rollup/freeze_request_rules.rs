use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze request must pass. Check returns its problems, or none.
pub trait FreezeRequestRule: std::fmt::Debug {
    fn check(&self, item: &FreezeRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRequestAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl FreezeRequestRule for FreezeRequestAllowedRegionRule {
    fn check(&self, item: &FreezeRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is duplicated", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRequestCodeLengthRule {
    pub max_length: i64,
}

impl FreezeRequestRule for FreezeRequestCodeLengthRule {
    fn check(&self, item: &FreezeRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRequestRegionLengthRule {
    pub max_length: i64,
}

impl FreezeRequestRule for FreezeRequestRegionLengthRule {
    fn check(&self, item: &FreezeRequest) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRequestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeRequestRule for FreezeRequestDurationMinutesRangeRule {
    fn check(&self, item: &FreezeRequest) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRequestRequiredLabelRule {
    pub message: String,
}

impl FreezeRequestRule for FreezeRequestRequiredLabelRule {
    fn check(&self, item: &FreezeRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRequestStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeRequestRule for FreezeRequestStartsAtRangeRule {
    fn check(&self, item: &FreezeRequest) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a freeze request.
#[derive(Debug, Clone)]
pub struct FreezeRequestValidator {
    pub rules: Vec<Rc<dyn FreezeRequestRule>>,
}

impl FreezeRequestValidator {
    pub fn validate(&self, item: &FreezeRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_request_validator() -> FreezeRequestValidator {
    FreezeRequestValidator {
        rules: vec![
            Rc::new(FreezeRequestAllowedRegionRule {
                allowed: vec!["tiered".to_string(), "seasonal".to_string()],
            }) as Rc<dyn FreezeRequestRule>,
            Rc::new(FreezeRequestCodeLengthRule { max_length: 80 }),
            Rc::new(FreezeRequestRegionLengthRule { max_length: 1000 }),
            Rc::new(FreezeRequestDurationMinutesRangeRule { min: 16, max: 1000 }),
            Rc::new(FreezeRequestRequiredLabelRule {
                message: "freeze request needs this field".to_string(),
            }),
            Rc::new(FreezeRequestStartsAtRangeRule { min: 1000, max: 86400 }),
        ],
    }
}
