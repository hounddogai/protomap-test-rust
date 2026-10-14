use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze audit must pass. Check returns its problems, or none.
pub trait FreezeAuditRule: std::fmt::Debug {
    fn check(&self, item: &FreezeAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAuditQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeAuditRule for FreezeAuditQuantityRangeRule {
    fn check(&self, item: &FreezeAudit) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAuditRequiredNameRule {
    pub message: String,
}

impl FreezeAuditRule for FreezeAuditRequiredNameRule {
    fn check(&self, item: &FreezeAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAuditCodeLengthRule {
    pub max_length: i64,
}

impl FreezeAuditRule for FreezeAuditCodeLengthRule {
    fn check(&self, item: &FreezeAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAuditAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FreezeAuditRule for FreezeAuditAllowedNameRule {
    fn check(&self, item: &FreezeAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze audit.
#[derive(Debug, Clone)]
pub struct FreezeAuditValidator {
    pub rules: Vec<Rc<dyn FreezeAuditRule>>,
}

impl FreezeAuditValidator {
    pub fn validate(&self, item: &FreezeAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_audit_validator() -> FreezeAuditValidator {
    FreezeAuditValidator {
        rules: vec![
            Rc::new(FreezeAuditQuantityRangeRule { min: 32, max: 3600 }) as Rc<dyn FreezeAuditRule>,
            Rc::new(FreezeAuditRequiredNameRule {
                message: "freeze audit needs this field".to_string(),
            }),
            Rc::new(FreezeAuditCodeLengthRule { max_length: 120 }),
            Rc::new(FreezeAuditAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}
