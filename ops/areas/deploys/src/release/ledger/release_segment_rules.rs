use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release segment must pass. Check returns its problems, or none.
pub trait ReleaseSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSegmentNameLengthRule {
    pub max_length: i64,
}

impl ReleaseSegmentRule for ReleaseSegmentNameLengthRule {
    fn check(&self, item: &ReleaseSegment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSegmentKeyIdLengthRule {
    pub max_length: i64,
}

impl ReleaseSegmentRule for ReleaseSegmentKeyIdLengthRule {
    fn check(&self, item: &ReleaseSegment) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSegmentSourceLengthRule {
    pub max_length: i64,
}

impl ReleaseSegmentRule for ReleaseSegmentSourceLengthRule {
    fn check(&self, item: &ReleaseSegment) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSegmentBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseSegmentRule for ReleaseSegmentBurstRangeRule {
    fn check(&self, item: &ReleaseSegment) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release segment.
#[derive(Debug, Clone)]
pub struct ReleaseSegmentValidator {
    pub rules: Vec<Rc<dyn ReleaseSegmentRule>>,
}

impl ReleaseSegmentValidator {
    pub fn validate(&self, item: &ReleaseSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_segment_validator() -> ReleaseSegmentValidator {
    ReleaseSegmentValidator {
        rules: vec![
            Rc::new(ReleaseSegmentNameLengthRule { max_length: 1000 }) as Rc<dyn ReleaseSegmentRule>,
            Rc::new(ReleaseSegmentKeyIdLengthRule { max_length: 86400 }),
            Rc::new(ReleaseSegmentSourceLengthRule { max_length: 86400 }),
            Rc::new(ReleaseSegmentBurstRangeRule { min: 16, max: 86400 }),
        ],
    }
}
