use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback entry must pass. Check returns its problems, or none.
pub trait RollbackEntryRule: std::fmt::Debug {
    fn check(&self, item: &RollbackEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEntryRequiredNameRule {
    pub message: String,
}

impl RollbackEntryRule for RollbackEntryRequiredNameRule {
    fn check(&self, item: &RollbackEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEntryNameLengthRule {
    pub max_length: i64,
}

impl RollbackEntryRule for RollbackEntryNameLengthRule {
    fn check(&self, item: &RollbackEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEntryFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackEntryRule for RollbackEntryFloorValueRangeRule {
    fn check(&self, item: &RollbackEntry) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEntryVisibleGuardRule {
    pub limit: f64,
}

impl RollbackEntryRule for RollbackEntryVisibleGuardRule {
    fn check(&self, item: &RollbackEntry) -> Vec<String> {
        if item.visible && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while visible",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEntryCodeLengthRule {
    pub max_length: i64,
}

impl RollbackEntryRule for RollbackEntryCodeLengthRule {
    fn check(&self, item: &RollbackEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEntryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollbackEntryRule for RollbackEntryAllowedCodeRule {
    fn check(&self, item: &RollbackEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback entry.
#[derive(Debug, Clone)]
pub struct RollbackEntryValidator {
    pub rules: Vec<Rc<dyn RollbackEntryRule>>,
}

impl RollbackEntryValidator {
    pub fn validate(&self, item: &RollbackEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_entry_validator() -> RollbackEntryValidator {
    RollbackEntryValidator {
        rules: vec![
            Rc::new(RollbackEntryRequiredNameRule {
                message: "rollback entry needs a value".to_string(),
            }) as Rc<dyn RollbackEntryRule>,
            Rc::new(RollbackEntryNameLengthRule { max_length: 255 }),
            Rc::new(RollbackEntryFloorValueRangeRule { min: 2.4, max: 2191.1 }),
            Rc::new(RollbackEntryVisibleGuardRule { limit: 0.0 }),
            Rc::new(RollbackEntryCodeLengthRule { max_length: 64 }),
            Rc::new(RollbackEntryAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "manual".to_string(), "tiered".to_string()],
            }),
        ],
    }
}
