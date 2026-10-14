use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering segment must pass. Check returns its problems, or none.
pub trait PeeringSegmentRule: std::fmt::Debug {
    fn check(&self, item: &PeeringSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSegmentRequiredCodeRule {
    pub message: String,
}

impl PeeringSegmentRule for PeeringSegmentRequiredCodeRule {
    fn check(&self, item: &PeeringSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSegmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl PeeringSegmentRule for PeeringSegmentOwnerIdLengthRule {
    fn check(&self, item: &PeeringSegment) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSegmentDescriptionLengthRule {
    pub max_length: i64,
}

impl PeeringSegmentRule for PeeringSegmentDescriptionLengthRule {
    fn check(&self, item: &PeeringSegment) -> Vec<String> {
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
pub struct PeeringSegmentSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringSegmentRule for PeeringSegmentSequenceRangeRule {
    fn check(&self, item: &PeeringSegment) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSegmentCodeLengthRule {
    pub max_length: i64,
}

impl PeeringSegmentRule for PeeringSegmentCodeLengthRule {
    fn check(&self, item: &PeeringSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSegmentMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringSegmentRule for PeeringSegmentMinimumRangeRule {
    fn check(&self, item: &PeeringSegment) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a peering segment.
#[derive(Debug, Clone)]
pub struct PeeringSegmentValidator {
    pub rules: Vec<Rc<dyn PeeringSegmentRule>>,
}

impl PeeringSegmentValidator {
    pub fn validate(&self, item: &PeeringSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_segment_validator() -> PeeringSegmentValidator {
    PeeringSegmentValidator {
        rules: vec![
            Rc::new(PeeringSegmentRequiredCodeRule {
                message: "peering segment needs this field".to_string(),
            }) as Rc<dyn PeeringSegmentRule>,
            Rc::new(PeeringSegmentOwnerIdLengthRule { max_length: 16 }),
            Rc::new(PeeringSegmentDescriptionLengthRule { max_length: 86400 }),
            Rc::new(PeeringSegmentSequenceRangeRule { min: 120, max: 1000 }),
            Rc::new(PeeringSegmentCodeLengthRule { max_length: 32 }),
            Rc::new(PeeringSegmentMinimumRangeRule { min: 1.7, max: 3434.1 }),
        ],
    }
}
