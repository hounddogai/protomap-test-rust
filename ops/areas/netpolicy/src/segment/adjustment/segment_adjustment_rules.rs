use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment adjustment must pass. Check returns its problems, or none.
pub trait SegmentAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &SegmentAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdjustmentRequiredCipherRule {
    pub message: String,
}

impl SegmentAdjustmentRule for SegmentAdjustmentRequiredCipherRule {
    fn check(&self, item: &SegmentAdjustment) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdjustmentRequiredCodeRule {
    pub message: String,
}

impl SegmentAdjustmentRule for SegmentAdjustmentRequiredCodeRule {
    fn check(&self, item: &SegmentAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdjustmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentAdjustmentRule for SegmentAdjustmentPortRangeRule {
    fn check(&self, item: &SegmentAdjustment) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdjustmentActiveGuardRule {
    pub limit: i64,
}

impl SegmentAdjustmentRule for SegmentAdjustmentActiveGuardRule {
    fn check(&self, item: &SegmentAdjustment) -> Vec<String> {
        if item.active && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while active", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdjustmentCipherLengthRule {
    pub max_length: i64,
}

impl SegmentAdjustmentRule for SegmentAdjustmentCipherLengthRule {
    fn check(&self, item: &SegmentAdjustment) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdjustmentRequiredNameRule {
    pub message: String,
}

impl SegmentAdjustmentRule for SegmentAdjustmentRequiredNameRule {
    fn check(&self, item: &SegmentAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment adjustment.
#[derive(Debug, Clone)]
pub struct SegmentAdjustmentValidator {
    pub rules: Vec<Rc<dyn SegmentAdjustmentRule>>,
}

impl SegmentAdjustmentValidator {
    pub fn validate(&self, item: &SegmentAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_adjustment_validator() -> SegmentAdjustmentValidator {
    SegmentAdjustmentValidator {
        rules: vec![
            Rc::new(SegmentAdjustmentRequiredCipherRule {
                message: "segment adjustment needs an entry".to_string(),
            }) as Rc<dyn SegmentAdjustmentRule>,
            Rc::new(SegmentAdjustmentRequiredCodeRule {
                message: "segment adjustment needs a value".to_string(),
            }),
            Rc::new(SegmentAdjustmentPortRangeRule { min: 255, max: 1000 }),
            Rc::new(SegmentAdjustmentActiveGuardRule { limit: 3600 }),
            Rc::new(SegmentAdjustmentCipherLengthRule { max_length: 255 }),
            Rc::new(SegmentAdjustmentRequiredNameRule {
                message: "segment adjustment needs this field".to_string(),
            }),
        ],
    }
}
