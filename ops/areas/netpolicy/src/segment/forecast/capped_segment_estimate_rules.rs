use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped segment estimate must pass. Check returns its problems, or none.
pub trait CappedSegmentEstimateRule: std::fmt::Debug {
    fn check(&self, item: &CappedSegmentEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentEstimateScopeLengthRule {
    pub max_length: i64,
}

impl CappedSegmentEstimateRule for CappedSegmentEstimateScopeLengthRule {
    fn check(&self, item: &CappedSegmentEstimate) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentEstimateRequiredLabelRule {
    pub message: String,
}

impl CappedSegmentEstimateRule for CappedSegmentEstimateRequiredLabelRule {
    fn check(&self, item: &CappedSegmentEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentEstimateRequiredRegionRule {
    pub message: String,
}

impl CappedSegmentEstimateRule for CappedSegmentEstimateRequiredRegionRule {
    fn check(&self, item: &CappedSegmentEstimate) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentEstimateRegionLengthRule {
    pub max_length: i64,
}

impl CappedSegmentEstimateRule for CappedSegmentEstimateRegionLengthRule {
    fn check(&self, item: &CappedSegmentEstimate) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped segment estimate.
#[derive(Debug, Clone)]
pub struct CappedSegmentEstimateValidator {
    pub rules: Vec<Rc<dyn CappedSegmentEstimateRule>>,
}

impl CappedSegmentEstimateValidator {
    pub fn validate(&self, item: &CappedSegmentEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedSegmentEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSegmentEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_segment_estimate_validator() -> CappedSegmentEstimateValidator {
    CappedSegmentEstimateValidator {
        rules: vec![
            Rc::new(CappedSegmentEstimateScopeLengthRule { max_length: 16 }) as Rc<dyn CappedSegmentEstimateRule>,
            Rc::new(CappedSegmentEstimateRequiredLabelRule {
                message: "capped segment estimate needs a value".to_string(),
            }),
            Rc::new(CappedSegmentEstimateRequiredRegionRule {
                message: "capped segment estimate needs this field".to_string(),
            }),
            Rc::new(CappedSegmentEstimateRegionLengthRule { max_length: 64 }),
        ],
    }
}
