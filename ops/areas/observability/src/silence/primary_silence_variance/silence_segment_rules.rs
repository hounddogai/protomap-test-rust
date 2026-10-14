use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence segment must pass. Check returns its problems, or none.
pub trait SilenceSegmentRule: std::fmt::Debug {
    fn check(&self, item: &SilenceSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegmentLabelLengthRule {
    pub max_length: i64,
}

impl SilenceSegmentRule for SilenceSegmentLabelLengthRule {
    fn check(&self, item: &SilenceSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceSegmentRule for SilenceSegmentDiscountRangeRule {
    fn check(&self, item: &SilenceSegment) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegmentRequiredAudienceRule {
    pub message: String,
}

impl SilenceSegmentRule for SilenceSegmentRequiredAudienceRule {
    fn check(&self, item: &SilenceSegment) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegmentRequiredCodeRule {
    pub message: String,
}

impl SilenceSegmentRule for SilenceSegmentRequiredCodeRule {
    fn check(&self, item: &SilenceSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegmentCodeLengthRule {
    pub max_length: i64,
}

impl SilenceSegmentRule for SilenceSegmentCodeLengthRule {
    fn check(&self, item: &SilenceSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceSegmentRule for SilenceSegmentVersionRangeRule {
    fn check(&self, item: &SilenceSegment) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence segment.
#[derive(Debug, Clone)]
pub struct SilenceSegmentValidator {
    pub rules: Vec<Rc<dyn SilenceSegmentRule>>,
}

impl SilenceSegmentValidator {
    pub fn validate(&self, item: &SilenceSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_segment_validator() -> SilenceSegmentValidator {
    SilenceSegmentValidator {
        rules: vec![
            Rc::new(SilenceSegmentLabelLengthRule { max_length: 32 }) as Rc<dyn SilenceSegmentRule>,
            Rc::new(SilenceSegmentDiscountRangeRule { min: 7.7, max: 2346.1 }),
            Rc::new(SilenceSegmentRequiredAudienceRule {
                message: "silence segment needs this field".to_string(),
            }),
            Rc::new(SilenceSegmentRequiredCodeRule {
                message: "silence segment needs an entry".to_string(),
            }),
            Rc::new(SilenceSegmentCodeLengthRule { max_length: 3600 }),
            Rc::new(SilenceSegmentVersionRangeRule { min: 64, max: 80 }),
        ],
    }
}
