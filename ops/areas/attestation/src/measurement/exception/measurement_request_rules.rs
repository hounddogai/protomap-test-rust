use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement request must pass. Check returns its problems, or none.
pub trait MeasurementRequestRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRequestPinnedGuardRule {
    pub limit: i64,
}

impl MeasurementRequestRule for MeasurementRequestPinnedGuardRule {
    fn check(&self, item: &MeasurementRequest) -> Vec<String> {
        if item.pinned && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while pinned",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRequestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl MeasurementRequestRule for MeasurementRequestAllowedCodeRule {
    fn check(&self, item: &MeasurementRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRequestRequiredCountryRule {
    pub message: String,
}

impl MeasurementRequestRule for MeasurementRequestRequiredCountryRule {
    fn check(&self, item: &MeasurementRequest) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRequestNameLengthRule {
    pub max_length: i64,
}

impl MeasurementRequestRule for MeasurementRequestNameLengthRule {
    fn check(&self, item: &MeasurementRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement request.
#[derive(Debug, Clone)]
pub struct MeasurementRequestValidator {
    pub rules: Vec<Rc<dyn MeasurementRequestRule>>,
}

impl MeasurementRequestValidator {
    pub fn validate(&self, item: &MeasurementRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_request_validator() -> MeasurementRequestValidator {
    MeasurementRequestValidator {
        rules: vec![
            Rc::new(MeasurementRequestPinnedGuardRule { limit: 32 }) as Rc<dyn MeasurementRequestRule>,
            Rc::new(MeasurementRequestAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(MeasurementRequestRequiredCountryRule {
                message: "measurement request needs a value".to_string(),
            }),
            Rc::new(MeasurementRequestNameLengthRule { max_length: 86400 }),
        ],
    }
}
