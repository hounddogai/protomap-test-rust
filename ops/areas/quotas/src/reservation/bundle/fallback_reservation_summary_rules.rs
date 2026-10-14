use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback reservation summary must pass. Check returns its problems, or none.
pub trait FallbackReservationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &FallbackReservationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationSummaryRequiredUnitRule {
    pub message: String,
}

impl FallbackReservationSummaryRule for FallbackReservationSummaryRequiredUnitRule {
    fn check(&self, item: &FallbackReservationSummary) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationSummaryPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackReservationSummaryRule for FallbackReservationSummaryPortRangeRule {
    fn check(&self, item: &FallbackReservationSummary) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationSummaryCodePrefixRule {
    pub prefix: String,
}

impl FallbackReservationSummaryRule for FallbackReservationSummaryCodePrefixRule {
    fn check(&self, item: &FallbackReservationSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback reservation summary.
#[derive(Debug, Clone)]
pub struct FallbackReservationSummaryValidator {
    pub rules: Vec<Rc<dyn FallbackReservationSummaryRule>>,
}

impl FallbackReservationSummaryValidator {
    pub fn validate(&self, item: &FallbackReservationSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackReservationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackReservationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_reservation_summary_validator() -> FallbackReservationSummaryValidator {
    FallbackReservationSummaryValidator {
        rules: vec![
            Rc::new(FallbackReservationSummaryRequiredUnitRule {
                message: "fallback reservation summary needs a value".to_string(),
            }) as Rc<dyn FallbackReservationSummaryRule>,
            Rc::new(FallbackReservationSummaryPortRangeRule { min: 16, max: 80 }),
            Rc::new(FallbackReservationSummaryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
