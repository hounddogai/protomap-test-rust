use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback alert cycle must pass. Check returns its problems, or none.
pub trait FallbackAlertCycleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackAlertCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertCycleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FallbackAlertCycleRule for FallbackAlertCycleAllowedNameRule {
    fn check(&self, item: &FallbackAlertCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertCycleArchivedGuardRule {
    pub limit: i64,
}

impl FallbackAlertCycleRule for FallbackAlertCycleArchivedGuardRule {
    fn check(&self, item: &FallbackAlertCycle) -> Vec<String> {
        if item.archived && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while archived",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertCycleRequiredCodeRule {
    pub message: String,
}

impl FallbackAlertCycleRule for FallbackAlertCycleRequiredCodeRule {
    fn check(&self, item: &FallbackAlertCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback alert cycle.
#[derive(Debug, Clone)]
pub struct FallbackAlertCycleValidator {
    pub rules: Vec<Rc<dyn FallbackAlertCycleRule>>,
}

impl FallbackAlertCycleValidator {
    pub fn validate(&self, item: &FallbackAlertCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackAlertCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackAlertCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_alert_cycle_validator() -> FallbackAlertCycleValidator {
    FallbackAlertCycleValidator {
        rules: vec![
            Rc::new(FallbackAlertCycleAllowedNameRule { allowed: Vec::new() }) as Rc<dyn FallbackAlertCycleRule>,
            Rc::new(FallbackAlertCycleArchivedGuardRule { limit: 80 }),
            Rc::new(FallbackAlertCycleRequiredCodeRule {
                message: "fallback alert cycle needs an entry".to_string(),
            }),
        ],
    }
}
