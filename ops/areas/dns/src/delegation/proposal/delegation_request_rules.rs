use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation request must pass. Check returns its problems, or none.
pub trait DelegationRequestRule: std::fmt::Debug {
    fn check(&self, item: &DelegationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRequestWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationRequestRule for DelegationRequestWindowDaysRangeRule {
    fn check(&self, item: &DelegationRequest) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRequestFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationRequestRule for DelegationRequestFloorValueRangeRule {
    fn check(&self, item: &DelegationRequest) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRequestRequiredCodeRule {
    pub message: String,
}

impl DelegationRequestRule for DelegationRequestRequiredCodeRule {
    fn check(&self, item: &DelegationRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRequestRequiredChecksumRule {
    pub message: String,
}

impl DelegationRequestRule for DelegationRequestRequiredChecksumRule {
    fn check(&self, item: &DelegationRequest) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRequestAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationRequestRule for DelegationRequestAttemptsRangeRule {
    fn check(&self, item: &DelegationRequest) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRequestExternalRefLengthRule {
    pub max_length: i64,
}

impl DelegationRequestRule for DelegationRequestExternalRefLengthRule {
    fn check(&self, item: &DelegationRequest) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation request.
#[derive(Debug, Clone)]
pub struct DelegationRequestValidator {
    pub rules: Vec<Rc<dyn DelegationRequestRule>>,
}

impl DelegationRequestValidator {
    pub fn validate(&self, item: &DelegationRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_request_validator() -> DelegationRequestValidator {
    DelegationRequestValidator {
        rules: vec![
            Rc::new(DelegationRequestWindowDaysRangeRule { min: 16, max: 32 }) as Rc<dyn DelegationRequestRule>,
            Rc::new(DelegationRequestFloorValueRangeRule { min: 3.7, max: 2186.7 }),
            Rc::new(DelegationRequestRequiredCodeRule {
                message: "delegation request needs an entry".to_string(),
            }),
            Rc::new(DelegationRequestRequiredChecksumRule {
                message: "delegation request needs a value".to_string(),
            }),
            Rc::new(DelegationRequestAttemptsRangeRule { min: 16, max: 3600 }),
            Rc::new(DelegationRequestExternalRefLengthRule { max_length: 120 }),
        ],
    }
}
