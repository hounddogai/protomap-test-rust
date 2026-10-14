use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity summary must pass. Check returns its problems, or none.
pub trait IdentitySummaryRule: std::fmt::Debug {
    fn check(&self, item: &IdentitySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySummaryRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentitySummaryRule for IdentitySummaryRotationDaysRangeRule {
    fn check(&self, item: &IdentitySummary) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySummaryCodeLengthRule {
    pub max_length: i64,
}

impl IdentitySummaryRule for IdentitySummaryCodeLengthRule {
    fn check(&self, item: &IdentitySummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySummaryRequiredCodeRule {
    pub message: String,
}

impl IdentitySummaryRule for IdentitySummaryRequiredCodeRule {
    fn check(&self, item: &IdentitySummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySummaryAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl IdentitySummaryRule for IdentitySummaryAllowedLabelRule {
    fn check(&self, item: &IdentitySummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity summary.
#[derive(Debug, Clone)]
pub struct IdentitySummaryValidator {
    pub rules: Vec<Rc<dyn IdentitySummaryRule>>,
}

impl IdentitySummaryValidator {
    pub fn validate(&self, item: &IdentitySummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentitySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentitySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_summary_validator() -> IdentitySummaryValidator {
    IdentitySummaryValidator {
        rules: vec![
            Rc::new(IdentitySummaryRotationDaysRangeRule { min: 120, max: 255 }) as Rc<dyn IdentitySummaryRule>,
            Rc::new(IdentitySummaryCodeLengthRule { max_length: 16 }),
            Rc::new(IdentitySummaryRequiredCodeRule {
                message: "identity summary needs an entry".to_string(),
            }),
            Rc::new(IdentitySummaryAllowedLabelRule {
                allowed: vec!["fallback".to_string(), "regional".to_string(), "batched".to_string()],
            }),
        ],
    }
}
