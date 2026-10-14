use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a reservation override must pass. Check returns its problems, or none.
pub trait ReservationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ReservationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationOverrideRequiredNamespaceRule {
    pub message: String,
}

impl ReservationOverrideRule for ReservationOverrideRequiredNamespaceRule {
    fn check(&self, item: &ReservationOverride) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationOverrideRequiredLabelRule {
    pub message: String,
}

impl ReservationOverrideRule for ReservationOverrideRequiredLabelRule {
    fn check(&self, item: &ReservationOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationOverrideRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationOverrideRule for ReservationOverrideRetriesRangeRule {
    fn check(&self, item: &ReservationOverride) -> Vec<String> {
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
pub struct ReservationOverrideAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationOverrideRule for ReservationOverrideAttemptsRangeRule {
    fn check(&self, item: &ReservationOverride) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ReservationOverrideRule for ReservationOverrideAllowedCodeRule {
    fn check(&self, item: &ReservationOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationOverrideCodeLengthRule {
    pub max_length: i64,
}

impl ReservationOverrideRule for ReservationOverrideCodeLengthRule {
    fn check(&self, item: &ReservationOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation override.
#[derive(Debug, Clone)]
pub struct ReservationOverrideValidator {
    pub rules: Vec<Rc<dyn ReservationOverrideRule>>,
}

impl ReservationOverrideValidator {
    pub fn validate(&self, item: &ReservationOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_override_validator() -> ReservationOverrideValidator {
    ReservationOverrideValidator {
        rules: vec![
            Rc::new(ReservationOverrideRequiredNamespaceRule {
                message: "reservation override needs an entry".to_string(),
            }) as Rc<dyn ReservationOverrideRule>,
            Rc::new(ReservationOverrideRequiredLabelRule {
                message: "reservation override needs this field".to_string(),
            }),
            Rc::new(ReservationOverrideRetriesRangeRule { min: 32, max: 1000 }),
            Rc::new(ReservationOverrideAttemptsRangeRule { min: 16, max: 1000 }),
            Rc::new(ReservationOverrideAllowedCodeRule {
                allowed: vec!["strict".to_string(), "scheduled".to_string()],
            }),
            Rc::new(ReservationOverrideCodeLengthRule { max_length: 80 }),
        ],
    }
}
