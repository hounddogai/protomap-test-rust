use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual redaction assignment must pass. Check returns its problems, or none.
pub trait ManualRedactionAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ManualRedactionAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionAssignmentRequiredLocaleRule {
    pub message: String,
}

impl ManualRedactionAssignmentRule for ManualRedactionAssignmentRequiredLocaleRule {
    fn check(&self, item: &ManualRedactionAssignment) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionAssignmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualRedactionAssignmentRule for ManualRedactionAssignmentWeightRangeRule {
    fn check(&self, item: &ManualRedactionAssignment) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualRedactionAssignmentRule for ManualRedactionAssignmentAllowedCodeRule {
    fn check(&self, item: &ManualRedactionAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual redaction assignment.
#[derive(Debug, Clone)]
pub struct ManualRedactionAssignmentValidator {
    pub rules: Vec<Rc<dyn ManualRedactionAssignmentRule>>,
}

impl ManualRedactionAssignmentValidator {
    pub fn validate(&self, item: &ManualRedactionAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualRedactionAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRedactionAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_redaction_assignment_validator() -> ManualRedactionAssignmentValidator {
    ManualRedactionAssignmentValidator {
        rules: vec![
            Rc::new(ManualRedactionAssignmentRequiredLocaleRule {
                message: "manual redaction assignment needs a value".to_string(),
            }) as Rc<dyn ManualRedactionAssignmentRule>,
            Rc::new(ManualRedactionAssignmentWeightRangeRule { min: 7.6, max: 597.1 }),
            Rc::new(ManualRedactionAssignmentAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}
