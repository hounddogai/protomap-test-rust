use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient peering draft must pass. Check returns its problems, or none.
pub trait LenientPeeringDraftRule: std::fmt::Debug {
    fn check(&self, item: &LenientPeeringDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringDraftCodeLengthRule {
    pub max_length: i64,
}

impl LenientPeeringDraftRule for LenientPeeringDraftCodeLengthRule {
    fn check(&self, item: &LenientPeeringDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringDraftPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPeeringDraftRule for LenientPeeringDraftPriorityRangeRule {
    fn check(&self, item: &LenientPeeringDraft) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringDraftAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LenientPeeringDraftRule for LenientPeeringDraftAllowedTitleRule {
    fn check(&self, item: &LenientPeeringDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringDraftRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPeeringDraftRule for LenientPeeringDraftRotationDaysRangeRule {
    fn check(&self, item: &LenientPeeringDraft) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient peering draft.
#[derive(Debug, Clone)]
pub struct LenientPeeringDraftValidator {
    pub rules: Vec<Rc<dyn LenientPeeringDraftRule>>,
}

impl LenientPeeringDraftValidator {
    pub fn validate(&self, item: &LenientPeeringDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPeeringDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPeeringDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_peering_draft_validator() -> LenientPeeringDraftValidator {
    LenientPeeringDraftValidator {
        rules: vec![
            Rc::new(LenientPeeringDraftCodeLengthRule { max_length: 32 }) as Rc<dyn LenientPeeringDraftRule>,
            Rc::new(LenientPeeringDraftPriorityRangeRule { min: 16, max: 120 }),
            Rc::new(LenientPeeringDraftAllowedTitleRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string()],
            }),
            Rc::new(LenientPeeringDraftRotationDaysRangeRule { min: 64, max: 80 }),
        ],
    }
}
