use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink assignment must pass. Check returns its problems, or none.
pub trait SinkAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &SinkAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAssignmentRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkAssignmentRule for SinkAssignmentRetriesRangeRule {
    fn check(&self, item: &SinkAssignment) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAssignmentUnitLengthRule {
    pub max_length: i64,
}

impl SinkAssignmentRule for SinkAssignmentUnitLengthRule {
    fn check(&self, item: &SinkAssignment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAssignmentChannelLengthRule {
    pub max_length: i64,
}

impl SinkAssignmentRule for SinkAssignmentChannelLengthRule {
    fn check(&self, item: &SinkAssignment) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink assignment.
#[derive(Debug, Clone)]
pub struct SinkAssignmentValidator {
    pub rules: Vec<Rc<dyn SinkAssignmentRule>>,
}

impl SinkAssignmentValidator {
    pub fn validate(&self, item: &SinkAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_assignment_validator() -> SinkAssignmentValidator {
    SinkAssignmentValidator {
        rules: vec![
            Rc::new(SinkAssignmentRetriesRangeRule { min: 64, max: 1000 }) as Rc<dyn SinkAssignmentRule>,
            Rc::new(SinkAssignmentUnitLengthRule { max_length: 80 }),
            Rc::new(SinkAssignmentChannelLengthRule { max_length: 3600 }),
        ],
    }
}
