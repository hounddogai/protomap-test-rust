use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary peering audit must pass. Check returns its problems, or none.
pub trait PrimaryPeeringAuditRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPeeringAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAuditAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PrimaryPeeringAuditRule for PrimaryPeeringAuditAllowedNameRule {
    fn check(&self, item: &PrimaryPeeringAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAuditWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryPeeringAuditRule for PrimaryPeeringAuditWindowDaysRangeRule {
    fn check(&self, item: &PrimaryPeeringAudit) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAuditPhoneLengthRule {
    pub max_length: i64,
}

impl PrimaryPeeringAuditRule for PrimaryPeeringAuditPhoneLengthRule {
    fn check(&self, item: &PrimaryPeeringAudit) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAuditAudienceLengthRule {
    pub max_length: i64,
}

impl PrimaryPeeringAuditRule for PrimaryPeeringAuditAudienceLengthRule {
    fn check(&self, item: &PrimaryPeeringAudit) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAuditRequiredNameRule {
    pub message: String,
}

impl PrimaryPeeringAuditRule for PrimaryPeeringAuditRequiredNameRule {
    fn check(&self, item: &PrimaryPeeringAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAuditBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryPeeringAuditRule for PrimaryPeeringAuditBatchSizeRangeRule {
    fn check(&self, item: &PrimaryPeeringAudit) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary peering audit.
#[derive(Debug, Clone)]
pub struct PrimaryPeeringAuditValidator {
    pub rules: Vec<Rc<dyn PrimaryPeeringAuditRule>>,
}

impl PrimaryPeeringAuditValidator {
    pub fn validate(&self, item: &PrimaryPeeringAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryPeeringAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPeeringAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_peering_audit_validator() -> PrimaryPeeringAuditValidator {
    PrimaryPeeringAuditValidator {
        rules: vec![
            Rc::new(PrimaryPeeringAuditAllowedNameRule { allowed: Vec::new() }) as Rc<dyn PrimaryPeeringAuditRule>,
            Rc::new(PrimaryPeeringAuditWindowDaysRangeRule { min: 3600, max: 3600 }),
            Rc::new(PrimaryPeeringAuditPhoneLengthRule { max_length: 120 }),
            Rc::new(PrimaryPeeringAuditAudienceLengthRule { max_length: 16 }),
            Rc::new(PrimaryPeeringAuditRequiredNameRule {
                message: "primary peering audit needs an entry".to_string(),
            }),
            Rc::new(PrimaryPeeringAuditBatchSizeRangeRule { min: 120, max: 1000 }),
        ],
    }
}
