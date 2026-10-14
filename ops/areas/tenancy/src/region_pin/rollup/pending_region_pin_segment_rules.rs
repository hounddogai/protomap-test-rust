use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending region pin segment must pass. Check returns its problems, or none.
pub trait PendingRegionPinSegmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingRegionPinSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinSegmentRequiredContactEmailRule {
    pub message: String,
}

impl PendingRegionPinSegmentRule for PendingRegionPinSegmentRequiredContactEmailRule {
    fn check(&self, item: &PendingRegionPinSegment) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinSegmentRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRegionPinSegmentRule for PendingRegionPinSegmentRetriesRangeRule {
    fn check(&self, item: &PendingRegionPinSegment) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinSegmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRegionPinSegmentRule for PendingRegionPinSegmentDiscountRangeRule {
    fn check(&self, item: &PendingRegionPinSegment) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinSegmentRequiredCodeRule {
    pub message: String,
}

impl PendingRegionPinSegmentRule for PendingRegionPinSegmentRequiredCodeRule {
    fn check(&self, item: &PendingRegionPinSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinSegmentLabelLengthRule {
    pub max_length: i64,
}

impl PendingRegionPinSegmentRule for PendingRegionPinSegmentLabelLengthRule {
    fn check(&self, item: &PendingRegionPinSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending region pin segment.
#[derive(Debug, Clone)]
pub struct PendingRegionPinSegmentValidator {
    pub rules: Vec<Rc<dyn PendingRegionPinSegmentRule>>,
}

impl PendingRegionPinSegmentValidator {
    pub fn validate(&self, item: &PendingRegionPinSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRegionPinSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRegionPinSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_region_pin_segment_validator() -> PendingRegionPinSegmentValidator {
    PendingRegionPinSegmentValidator {
        rules: vec![
            Rc::new(PendingRegionPinSegmentRequiredContactEmailRule {
                message: "pending region pin segment needs a value".to_string(),
            }) as Rc<dyn PendingRegionPinSegmentRule>,
            Rc::new(PendingRegionPinSegmentRetriesRangeRule { min: 255, max: 86400 }),
            Rc::new(PendingRegionPinSegmentDiscountRangeRule { min: 6.0, max: 709.5 }),
            Rc::new(PendingRegionPinSegmentRequiredCodeRule {
                message: "pending region pin segment needs this field".to_string(),
            }),
            Rc::new(PendingRegionPinSegmentLabelLengthRule { max_length: 120 }),
        ],
    }
}
