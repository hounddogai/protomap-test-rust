use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict reservation segment must pass. Check returns its problems, or none.
pub trait StrictReservationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &StrictReservationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationSegmentRequiredNameRule {
    pub message: String,
}

impl StrictReservationSegmentRule for StrictReservationSegmentRequiredNameRule {
    fn check(&self, item: &StrictReservationSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationSegmentCodePrefixRule {
    pub prefix: String,
}

impl StrictReservationSegmentRule for StrictReservationSegmentCodePrefixRule {
    fn check(&self, item: &StrictReservationSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationSegmentTitleLengthRule {
    pub max_length: i64,
}

impl StrictReservationSegmentRule for StrictReservationSegmentTitleLengthRule {
    fn check(&self, item: &StrictReservationSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationSegmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictReservationSegmentRule for StrictReservationSegmentWeightRangeRule {
    fn check(&self, item: &StrictReservationSegment) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict reservation segment.
#[derive(Debug, Clone)]
pub struct StrictReservationSegmentValidator {
    pub rules: Vec<Rc<dyn StrictReservationSegmentRule>>,
}

impl StrictReservationSegmentValidator {
    pub fn validate(&self, item: &StrictReservationSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictReservationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictReservationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_reservation_segment_validator() -> StrictReservationSegmentValidator {
    StrictReservationSegmentValidator {
        rules: vec![
            Rc::new(StrictReservationSegmentRequiredNameRule {
                message: "strict reservation segment needs this field".to_string(),
            }) as Rc<dyn StrictReservationSegmentRule>,
            Rc::new(StrictReservationSegmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(StrictReservationSegmentTitleLengthRule { max_length: 120 }),
            Rc::new(StrictReservationSegmentWeightRangeRule { min: 9.4, max: 3427.8 }),
        ],
    }
}
