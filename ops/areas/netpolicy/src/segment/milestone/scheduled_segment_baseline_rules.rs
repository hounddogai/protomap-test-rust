use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled segment baseline must pass. Check returns its problems, or none.
pub trait ScheduledSegmentBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSegmentBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentBaselineOwnerIdLengthRule {
    pub max_length: i64,
}

impl ScheduledSegmentBaselineRule for ScheduledSegmentBaselineOwnerIdLengthRule {
    fn check(&self, item: &ScheduledSegmentBaseline) -> Vec<String> {
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
pub struct ScheduledSegmentBaselinePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSegmentBaselineRule for ScheduledSegmentBaselinePortRangeRule {
    fn check(&self, item: &ScheduledSegmentBaseline) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentBaselinePhoneLengthRule {
    pub max_length: i64,
}

impl ScheduledSegmentBaselineRule for ScheduledSegmentBaselinePhoneLengthRule {
    fn check(&self, item: &ScheduledSegmentBaseline) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentBaselineRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledSegmentBaselineRule for ScheduledSegmentBaselineRatioRangeRule {
    fn check(&self, item: &ScheduledSegmentBaseline) -> Vec<String> {
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

/// Runs every rule of a scheduled segment baseline.
#[derive(Debug, Clone)]
pub struct ScheduledSegmentBaselineValidator {
    pub rules: Vec<Rc<dyn ScheduledSegmentBaselineRule>>,
}

impl ScheduledSegmentBaselineValidator {
    pub fn validate(&self, item: &ScheduledSegmentBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledSegmentBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSegmentBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_segment_baseline_validator() -> ScheduledSegmentBaselineValidator {
    ScheduledSegmentBaselineValidator {
        rules: vec![
            Rc::new(ScheduledSegmentBaselineOwnerIdLengthRule { max_length: 64 })
                as Rc<dyn ScheduledSegmentBaselineRule>,
            Rc::new(ScheduledSegmentBaselinePortRangeRule { min: 120, max: 255 }),
            Rc::new(ScheduledSegmentBaselinePhoneLengthRule { max_length: 80 }),
            Rc::new(ScheduledSegmentBaselineRatioRangeRule { min: 6.9, max: 3194.2 }),
        ],
    }
}
