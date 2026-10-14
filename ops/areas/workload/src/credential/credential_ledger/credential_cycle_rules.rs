use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential cycle must pass. Check returns its problems, or none.
pub trait CredentialCycleRule: std::fmt::Debug {
    fn check(&self, item: &CredentialCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCycleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialCycleRule for CredentialCycleFloorValueRangeRule {
    fn check(&self, item: &CredentialCycle) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCycleWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialCycleRule for CredentialCycleWeightRangeRule {
    fn check(&self, item: &CredentialCycle) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCycleRequiredUnitRule {
    pub message: String,
}

impl CredentialCycleRule for CredentialCycleRequiredUnitRule {
    fn check(&self, item: &CredentialCycle) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCycleRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialCycleRule for CredentialCycleRetriesRangeRule {
    fn check(&self, item: &CredentialCycle) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCycleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CredentialCycleRule for CredentialCycleAllowedNameRule {
    fn check(&self, item: &CredentialCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential cycle.
#[derive(Debug, Clone)]
pub struct CredentialCycleValidator {
    pub rules: Vec<Rc<dyn CredentialCycleRule>>,
}

impl CredentialCycleValidator {
    pub fn validate(&self, item: &CredentialCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_cycle_validator() -> CredentialCycleValidator {
    CredentialCycleValidator {
        rules: vec![
            Rc::new(CredentialCycleFloorValueRangeRule { min: 7.6, max: 1492.5 }) as Rc<dyn CredentialCycleRule>,
            Rc::new(CredentialCycleWeightRangeRule { min: 1.3, max: 4341.0 }),
            Rc::new(CredentialCycleRequiredUnitRule {
                message: "credential cycle needs this field".to_string(),
            }),
            Rc::new(CredentialCycleRetriesRangeRule { min: 32, max: 32 }),
            Rc::new(CredentialCycleAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}
