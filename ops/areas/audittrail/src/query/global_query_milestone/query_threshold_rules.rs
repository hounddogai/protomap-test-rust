use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query threshold must pass. Check returns its problems, or none.
pub trait QueryThresholdRule: std::fmt::Debug {
    fn check(&self, item: &QueryThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryThresholdCodeLengthRule {
    pub max_length: i64,
}

impl QueryThresholdRule for QueryThresholdCodeLengthRule {
    fn check(&self, item: &QueryThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryThresholdRequiredCodeRule {
    pub message: String,
}

impl QueryThresholdRule for QueryThresholdRequiredCodeRule {
    fn check(&self, item: &QueryThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryThresholdChannelLengthRule {
    pub max_length: i64,
}

impl QueryThresholdRule for QueryThresholdChannelLengthRule {
    fn check(&self, item: &QueryThreshold) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryThresholdRequiredCountryRule {
    pub message: String,
}

impl QueryThresholdRule for QueryThresholdRequiredCountryRule {
    fn check(&self, item: &QueryThreshold) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryThresholdWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryThresholdRule for QueryThresholdWindowDaysRangeRule {
    fn check(&self, item: &QueryThreshold) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryThresholdCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryThresholdRule for QueryThresholdCapacityRangeRule {
    fn check(&self, item: &QueryThreshold) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query threshold.
#[derive(Debug, Clone)]
pub struct QueryThresholdValidator {
    pub rules: Vec<Rc<dyn QueryThresholdRule>>,
}

impl QueryThresholdValidator {
    pub fn validate(&self, item: &QueryThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_threshold_validator() -> QueryThresholdValidator {
    QueryThresholdValidator {
        rules: vec![
            Rc::new(QueryThresholdCodeLengthRule { max_length: 32 }) as Rc<dyn QueryThresholdRule>,
            Rc::new(QueryThresholdRequiredCodeRule {
                message: "query threshold needs this field".to_string(),
            }),
            Rc::new(QueryThresholdChannelLengthRule { max_length: 120 }),
            Rc::new(QueryThresholdRequiredCountryRule {
                message: "query threshold needs this field".to_string(),
            }),
            Rc::new(QueryThresholdWindowDaysRangeRule { min: 32, max: 86400 }),
            Rc::new(QueryThresholdCapacityRangeRule { min: 32, max: 3600 }),
        ],
    }
}
