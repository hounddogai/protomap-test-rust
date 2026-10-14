use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled reservation override must pass. Check returns its problems, or none.
pub trait ScheduledReservationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledReservationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationOverrideCodePrefixRule {
    pub prefix: String,
}

impl ScheduledReservationOverrideRule for ScheduledReservationOverrideCodePrefixRule {
    fn check(&self, item: &ScheduledReservationOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationOverrideLocaleLengthRule {
    pub max_length: i64,
}

impl ScheduledReservationOverrideRule for ScheduledReservationOverrideLocaleLengthRule {
    fn check(&self, item: &ScheduledReservationOverride) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationOverrideRequiredCodeRule {
    pub message: String,
}

impl ScheduledReservationOverrideRule for ScheduledReservationOverrideRequiredCodeRule {
    fn check(&self, item: &ScheduledReservationOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled reservation override.
#[derive(Debug, Clone)]
pub struct ScheduledReservationOverrideValidator {
    pub rules: Vec<Rc<dyn ScheduledReservationOverrideRule>>,
}

impl ScheduledReservationOverrideValidator {
    pub fn validate(&self, item: &ScheduledReservationOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledReservationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledReservationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_reservation_override_validator() -> ScheduledReservationOverrideValidator {
    ScheduledReservationOverrideValidator {
        rules: vec![
            Rc::new(ScheduledReservationOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn ScheduledReservationOverrideRule>,
            Rc::new(ScheduledReservationOverrideLocaleLengthRule { max_length: 80 }),
            Rc::new(ScheduledReservationOverrideRequiredCodeRule {
                message: "scheduled reservation override needs this field".to_string(),
            }),
        ],
    }
}
