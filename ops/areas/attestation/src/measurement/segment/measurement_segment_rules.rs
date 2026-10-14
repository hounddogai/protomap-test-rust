use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement segment must pass. Check returns its problems, or none.
pub trait MeasurementSegmentRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSegmentRequiredNameRule {
    pub message: String,
}

impl MeasurementSegmentRule for MeasurementSegmentRequiredNameRule {
    fn check(&self, item: &MeasurementSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSegmentNameLengthRule {
    pub max_length: i64,
}

impl MeasurementSegmentRule for MeasurementSegmentNameLengthRule {
    fn check(&self, item: &MeasurementSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSegmentCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementSegmentRule for MeasurementSegmentCodeLengthRule {
    fn check(&self, item: &MeasurementSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSegmentTitleLengthRule {
    pub max_length: i64,
}

impl MeasurementSegmentRule for MeasurementSegmentTitleLengthRule {
    fn check(&self, item: &MeasurementSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSegmentCodePrefixRule {
    pub prefix: String,
}

impl MeasurementSegmentRule for MeasurementSegmentCodePrefixRule {
    fn check(&self, item: &MeasurementSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement segment.
#[derive(Debug, Clone)]
pub struct MeasurementSegmentValidator {
    pub rules: Vec<Rc<dyn MeasurementSegmentRule>>,
}

impl MeasurementSegmentValidator {
    pub fn validate(&self, item: &MeasurementSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_segment_validator() -> MeasurementSegmentValidator {
    MeasurementSegmentValidator {
        rules: vec![
            Rc::new(MeasurementSegmentRequiredNameRule {
                message: "measurement segment needs this field".to_string(),
            }) as Rc<dyn MeasurementSegmentRule>,
            Rc::new(MeasurementSegmentNameLengthRule { max_length: 255 }),
            Rc::new(MeasurementSegmentCodeLengthRule { max_length: 16 }),
            Rc::new(MeasurementSegmentTitleLengthRule { max_length: 120 }),
            Rc::new(MeasurementSegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
