use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal health check segment must pass. Check returns its problems, or none.
pub trait SeasonalHealthCheckSegmentRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalHealthCheckSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckSegmentClusterLengthRule {
    pub max_length: i64,
}

impl SeasonalHealthCheckSegmentRule for SeasonalHealthCheckSegmentClusterLengthRule {
    fn check(&self, item: &SeasonalHealthCheckSegment) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckSegmentRequiredClusterRule {
    pub message: String,
}

impl SeasonalHealthCheckSegmentRule for SeasonalHealthCheckSegmentRequiredClusterRule {
    fn check(&self, item: &SeasonalHealthCheckSegment) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckSegmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalHealthCheckSegmentRule for SeasonalHealthCheckSegmentLimitRangeRule {
    fn check(&self, item: &SeasonalHealthCheckSegment) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckSegmentRequiredCodeRule {
    pub message: String,
}

impl SeasonalHealthCheckSegmentRule for SeasonalHealthCheckSegmentRequiredCodeRule {
    fn check(&self, item: &SeasonalHealthCheckSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckSegmentCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalHealthCheckSegmentRule for SeasonalHealthCheckSegmentCodeLengthRule {
    fn check(&self, item: &SeasonalHealthCheckSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal health check segment.
#[derive(Debug, Clone)]
pub struct SeasonalHealthCheckSegmentValidator {
    pub rules: Vec<Rc<dyn SeasonalHealthCheckSegmentRule>>,
}

impl SeasonalHealthCheckSegmentValidator {
    pub fn validate(&self, item: &SeasonalHealthCheckSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalHealthCheckSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalHealthCheckSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_health_check_segment_validator() -> SeasonalHealthCheckSegmentValidator {
    SeasonalHealthCheckSegmentValidator {
        rules: vec![
            Rc::new(SeasonalHealthCheckSegmentClusterLengthRule { max_length: 64 })
                as Rc<dyn SeasonalHealthCheckSegmentRule>,
            Rc::new(SeasonalHealthCheckSegmentRequiredClusterRule {
                message: "seasonal health check segment needs a value".to_string(),
            }),
            Rc::new(SeasonalHealthCheckSegmentLimitRangeRule { min: 32, max: 64 }),
            Rc::new(SeasonalHealthCheckSegmentRequiredCodeRule {
                message: "seasonal health check segment needs a value".to_string(),
            }),
            Rc::new(SeasonalHealthCheckSegmentCodeLengthRule { max_length: 32 }),
        ],
    }
}
