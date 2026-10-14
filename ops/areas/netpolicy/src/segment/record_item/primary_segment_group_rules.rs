use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary segment group must pass. Check returns its problems, or none.
pub trait PrimarySegmentGroupRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySegmentGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySegmentGroupRequiredAudienceRule {
    pub message: String,
}

impl PrimarySegmentGroupRule for PrimarySegmentGroupRequiredAudienceRule {
    fn check(&self, item: &PrimarySegmentGroup) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySegmentGroupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimarySegmentGroupRule for PrimarySegmentGroupAllowedCodeRule {
    fn check(&self, item: &PrimarySegmentGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySegmentGroupAudienceLengthRule {
    pub max_length: i64,
}

impl PrimarySegmentGroupRule for PrimarySegmentGroupAudienceLengthRule {
    fn check(&self, item: &PrimarySegmentGroup) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySegmentGroupRequiredLabelRule {
    pub message: String,
}

impl PrimarySegmentGroupRule for PrimarySegmentGroupRequiredLabelRule {
    fn check(&self, item: &PrimarySegmentGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary segment group.
#[derive(Debug, Clone)]
pub struct PrimarySegmentGroupValidator {
    pub rules: Vec<Rc<dyn PrimarySegmentGroupRule>>,
}

impl PrimarySegmentGroupValidator {
    pub fn validate(&self, item: &PrimarySegmentGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySegmentGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySegmentGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_segment_group_validator() -> PrimarySegmentGroupValidator {
    PrimarySegmentGroupValidator {
        rules: vec![
            Rc::new(PrimarySegmentGroupRequiredAudienceRule {
                message: "primary segment group needs this field".to_string(),
            }) as Rc<dyn PrimarySegmentGroupRule>,
            Rc::new(PrimarySegmentGroupAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PrimarySegmentGroupAudienceLengthRule { max_length: 1000 }),
            Rc::new(PrimarySegmentGroupRequiredLabelRule {
                message: "primary segment group needs an entry".to_string(),
            }),
        ],
    }
}
