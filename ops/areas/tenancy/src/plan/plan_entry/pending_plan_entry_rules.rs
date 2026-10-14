use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending plan entry must pass. Check returns its problems, or none.
pub trait PendingPlanEntryRule: std::fmt::Debug {
    fn check(&self, item: &PendingPlanEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanEntryAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingPlanEntryRule for PendingPlanEntryAllowedLabelRule {
    fn check(&self, item: &PendingPlanEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanEntryMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingPlanEntryRule for PendingPlanEntryMinimumRangeRule {
    fn check(&self, item: &PendingPlanEntry) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanEntryCodeLengthRule {
    pub max_length: i64,
}

impl PendingPlanEntryRule for PendingPlanEntryCodeLengthRule {
    fn check(&self, item: &PendingPlanEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanEntryCodePrefixRule {
    pub prefix: String,
}

impl PendingPlanEntryRule for PendingPlanEntryCodePrefixRule {
    fn check(&self, item: &PendingPlanEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending plan entry.
#[derive(Debug, Clone)]
pub struct PendingPlanEntryValidator {
    pub rules: Vec<Rc<dyn PendingPlanEntryRule>>,
}

impl PendingPlanEntryValidator {
    pub fn validate(&self, item: &PendingPlanEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPlanEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPlanEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_plan_entry_validator() -> PendingPlanEntryValidator {
    PendingPlanEntryValidator {
        rules: vec![
            Rc::new(PendingPlanEntryAllowedLabelRule {
                allowed: vec!["legacy".to_string(), "seasonal".to_string(), "batched".to_string()],
            }) as Rc<dyn PendingPlanEntryRule>,
            Rc::new(PendingPlanEntryMinimumRangeRule { min: 2.2, max: 4904.6 }),
            Rc::new(PendingPlanEntryCodeLengthRule { max_length: 3600 }),
            Rc::new(PendingPlanEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
