use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline draft must pass. Check returns its problems, or none.
pub trait TimelineDraftRule: std::fmt::Debug {
    fn check(&self, item: &TimelineDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDraftRequiredNameRule {
    pub message: String,
}

impl TimelineDraftRule for TimelineDraftRequiredNameRule {
    fn check(&self, item: &TimelineDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TimelineDraftRule for TimelineDraftAllowedCodeRule {
    fn check(&self, item: &TimelineDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDraftRequiredAudienceRule {
    pub message: String,
}

impl TimelineDraftRule for TimelineDraftRequiredAudienceRule {
    fn check(&self, item: &TimelineDraft) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDraftRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineDraftRule for TimelineDraftRateRangeRule {
    fn check(&self, item: &TimelineDraft) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDraftKeyIdLengthRule {
    pub max_length: i64,
}

impl TimelineDraftRule for TimelineDraftKeyIdLengthRule {
    fn check(&self, item: &TimelineDraft) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline draft.
#[derive(Debug, Clone)]
pub struct TimelineDraftValidator {
    pub rules: Vec<Rc<dyn TimelineDraftRule>>,
}

impl TimelineDraftValidator {
    pub fn validate(&self, item: &TimelineDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_draft_validator() -> TimelineDraftValidator {
    TimelineDraftValidator {
        rules: vec![
            Rc::new(TimelineDraftRequiredNameRule {
                message: "timeline draft needs this field".to_string(),
            }) as Rc<dyn TimelineDraftRule>,
            Rc::new(TimelineDraftAllowedCodeRule {
                allowed: vec!["capped".to_string(), "capped".to_string()],
            }),
            Rc::new(TimelineDraftRequiredAudienceRule {
                message: "timeline draft needs a value".to_string(),
            }),
            Rc::new(TimelineDraftRateRangeRule { min: 7.4, max: 2765.8 }),
            Rc::new(TimelineDraftKeyIdLengthRule { max_length: 1000 }),
        ],
    }
}
