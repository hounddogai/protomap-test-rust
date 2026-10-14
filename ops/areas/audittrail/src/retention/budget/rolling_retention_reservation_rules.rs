use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling retention reservation must pass. Check returns its problems, or none.
pub trait RollingRetentionReservationRule: std::fmt::Debug {
    fn check(&self, item: &RollingRetentionReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRetentionReservationScopeLengthRule {
    pub max_length: i64,
}

impl RollingRetentionReservationRule for RollingRetentionReservationScopeLengthRule {
    fn check(&self, item: &RollingRetentionReservation) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRetentionReservationCodeLengthRule {
    pub max_length: i64,
}

impl RollingRetentionReservationRule for RollingRetentionReservationCodeLengthRule {
    fn check(&self, item: &RollingRetentionReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRetentionReservationCodePrefixRule {
    pub prefix: String,
}

impl RollingRetentionReservationRule for RollingRetentionReservationCodePrefixRule {
    fn check(&self, item: &RollingRetentionReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling retention reservation.
#[derive(Debug, Clone)]
pub struct RollingRetentionReservationValidator {
    pub rules: Vec<Rc<dyn RollingRetentionReservationRule>>,
}

impl RollingRetentionReservationValidator {
    pub fn validate(&self, item: &RollingRetentionReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingRetentionReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRetentionReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_retention_reservation_validator() -> RollingRetentionReservationValidator {
    RollingRetentionReservationValidator {
        rules: vec![
            Rc::new(RollingRetentionReservationScopeLengthRule { max_length: 32 })
                as Rc<dyn RollingRetentionReservationRule>,
            Rc::new(RollingRetentionReservationCodeLengthRule { max_length: 64 }),
            Rc::new(RollingRetentionReservationCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
