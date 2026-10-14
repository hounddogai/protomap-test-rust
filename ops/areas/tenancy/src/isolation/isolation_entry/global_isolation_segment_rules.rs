use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global isolation segment must pass. Check returns its problems, or none.
pub trait GlobalIsolationSegmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalIsolationSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationSegmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalIsolationSegmentRule for GlobalIsolationSegmentDiscountRangeRule {
    fn check(&self, item: &GlobalIsolationSegment) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationSegmentLabelLengthRule {
    pub max_length: i64,
}

impl GlobalIsolationSegmentRule for GlobalIsolationSegmentLabelLengthRule {
    fn check(&self, item: &GlobalIsolationSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationSegmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalIsolationSegmentRule for GlobalIsolationSegmentWeightRangeRule {
    fn check(&self, item: &GlobalIsolationSegment) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global isolation segment.
#[derive(Debug, Clone)]
pub struct GlobalIsolationSegmentValidator {
    pub rules: Vec<Rc<dyn GlobalIsolationSegmentRule>>,
}

impl GlobalIsolationSegmentValidator {
    pub fn validate(&self, item: &GlobalIsolationSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalIsolationSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalIsolationSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_isolation_segment_validator() -> GlobalIsolationSegmentValidator {
    GlobalIsolationSegmentValidator {
        rules: vec![
            Rc::new(GlobalIsolationSegmentDiscountRangeRule { min: 6.2, max: 4249.3 })
                as Rc<dyn GlobalIsolationSegmentRule>,
            Rc::new(GlobalIsolationSegmentLabelLengthRule { max_length: 3600 }),
            Rc::new(GlobalIsolationSegmentWeightRangeRule { min: 0.8, max: 4758.0 }),
        ],
    }
}
