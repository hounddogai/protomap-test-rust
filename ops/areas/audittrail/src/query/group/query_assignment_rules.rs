use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query assignment must pass. Check returns its problems, or none.
pub trait QueryAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &QueryAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAssignmentRequiredChecksumRule {
    pub message: String,
}

impl QueryAssignmentRule for QueryAssignmentRequiredChecksumRule {
    fn check(&self, item: &QueryAssignment) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAssignmentAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl QueryAssignmentRule for QueryAssignmentAllowedPhoneRule {
    fn check(&self, item: &QueryAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is out of range", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl QueryAssignmentRule for QueryAssignmentCodeLengthRule {
    fn check(&self, item: &QueryAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAssignmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryAssignmentRule for QueryAssignmentPortRangeRule {
    fn check(&self, item: &QueryAssignment) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAssignmentRequiredCodeRule {
    pub message: String,
}

impl QueryAssignmentRule for QueryAssignmentRequiredCodeRule {
    fn check(&self, item: &QueryAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryAssignmentBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryAssignmentRule for QueryAssignmentBatchSizeRangeRule {
    fn check(&self, item: &QueryAssignment) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query assignment.
#[derive(Debug, Clone)]
pub struct QueryAssignmentValidator {
    pub rules: Vec<Rc<dyn QueryAssignmentRule>>,
}

impl QueryAssignmentValidator {
    pub fn validate(&self, item: &QueryAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_assignment_validator() -> QueryAssignmentValidator {
    QueryAssignmentValidator {
        rules: vec![
            Rc::new(QueryAssignmentRequiredChecksumRule {
                message: "query assignment needs a value".to_string(),
            }) as Rc<dyn QueryAssignmentRule>,
            Rc::new(QueryAssignmentAllowedPhoneRule {
                allowed: vec!["primary".to_string(), "fallback".to_string()],
            }),
            Rc::new(QueryAssignmentCodeLengthRule { max_length: 16 }),
            Rc::new(QueryAssignmentPortRangeRule { min: 16, max: 86400 }),
            Rc::new(QueryAssignmentRequiredCodeRule {
                message: "query assignment needs an entry".to_string(),
            }),
            Rc::new(QueryAssignmentBatchSizeRangeRule { min: 1000, max: 86400 }),
        ],
    }
}
