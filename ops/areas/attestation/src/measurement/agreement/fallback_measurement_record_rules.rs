use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback measurement record must pass. Check returns its problems, or none.
pub trait FallbackMeasurementRecordRule: std::fmt::Debug {
    fn check(&self, item: &FallbackMeasurementRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementRecordRequiredSegmentRule {
    pub message: String,
}

impl FallbackMeasurementRecordRule for FallbackMeasurementRecordRequiredSegmentRule {
    fn check(&self, item: &FallbackMeasurementRecord) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementRecordSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackMeasurementRecordRule for FallbackMeasurementRecordSequenceRangeRule {
    fn check(&self, item: &FallbackMeasurementRecord) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementRecordStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackMeasurementRecordRule for FallbackMeasurementRecordStartsAtRangeRule {
    fn check(&self, item: &FallbackMeasurementRecord) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementRecordRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackMeasurementRecordRule for FallbackMeasurementRecordRatioRangeRule {
    fn check(&self, item: &FallbackMeasurementRecord) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementRecordRequiredCodeRule {
    pub message: String,
}

impl FallbackMeasurementRecordRule for FallbackMeasurementRecordRequiredCodeRule {
    fn check(&self, item: &FallbackMeasurementRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementRecordRequiredTitleRule {
    pub message: String,
}

impl FallbackMeasurementRecordRule for FallbackMeasurementRecordRequiredTitleRule {
    fn check(&self, item: &FallbackMeasurementRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback measurement record.
#[derive(Debug, Clone)]
pub struct FallbackMeasurementRecordValidator {
    pub rules: Vec<Rc<dyn FallbackMeasurementRecordRule>>,
}

impl FallbackMeasurementRecordValidator {
    pub fn validate(&self, item: &FallbackMeasurementRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackMeasurementRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackMeasurementRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_measurement_record_validator() -> FallbackMeasurementRecordValidator {
    FallbackMeasurementRecordValidator {
        rules: vec![
            Rc::new(FallbackMeasurementRecordRequiredSegmentRule {
                message: "fallback measurement record needs this field".to_string(),
            }) as Rc<dyn FallbackMeasurementRecordRule>,
            Rc::new(FallbackMeasurementRecordSequenceRangeRule { min: 16, max: 120 }),
            Rc::new(FallbackMeasurementRecordStartsAtRangeRule { min: 255, max: 3600 }),
            Rc::new(FallbackMeasurementRecordRatioRangeRule { min: 1.9, max: 750.7 }),
            Rc::new(FallbackMeasurementRecordRequiredCodeRule {
                message: "fallback measurement record needs this field".to_string(),
            }),
            Rc::new(FallbackMeasurementRecordRequiredTitleRule {
                message: "fallback measurement record needs this field".to_string(),
            }),
        ],
    }
}
