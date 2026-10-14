use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin summary must pass. Check returns its problems, or none.
pub trait RegionPinSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSummaryRequiredCodeRule {
    pub message: String,
}

impl RegionPinSummaryRule for RegionPinSummaryRequiredCodeRule {
    fn check(&self, item: &RegionPinSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSummaryDescriptionLengthRule {
    pub max_length: i64,
}

impl RegionPinSummaryRule for RegionPinSummaryDescriptionLengthRule {
    fn check(&self, item: &RegionPinSummary) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionPinSummaryRule for RegionPinSummaryAllowedCodeRule {
    fn check(&self, item: &RegionPinSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSummaryLabelLengthRule {
    pub max_length: i64,
}

impl RegionPinSummaryRule for RegionPinSummaryLabelLengthRule {
    fn check(&self, item: &RegionPinSummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin summary.
#[derive(Debug, Clone)]
pub struct RegionPinSummaryValidator {
    pub rules: Vec<Rc<dyn RegionPinSummaryRule>>,
}

impl RegionPinSummaryValidator {
    pub fn validate(&self, item: &RegionPinSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_summary_validator() -> RegionPinSummaryValidator {
    RegionPinSummaryValidator {
        rules: vec![
            Rc::new(RegionPinSummaryRequiredCodeRule {
                message: "region pin summary needs an entry".to_string(),
            }) as Rc<dyn RegionPinSummaryRule>,
            Rc::new(RegionPinSummaryDescriptionLengthRule { max_length: 80 }),
            Rc::new(RegionPinSummaryAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(RegionPinSummaryLabelLengthRule { max_length: 64 }),
        ],
    }
}
