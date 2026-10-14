use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query segment must pass. Check returns its problems, or none.
pub trait QuerySegmentRule: std::fmt::Debug {
    fn check(&self, item: &QuerySegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySegmentCodePrefixRule {
    pub prefix: String,
}

impl QuerySegmentRule for QuerySegmentCodePrefixRule {
    fn check(&self, item: &QuerySegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySegmentRequiredLabelRule {
    pub message: String,
}

impl QuerySegmentRule for QuerySegmentRequiredLabelRule {
    fn check(&self, item: &QuerySegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySegmentCodeLengthRule {
    pub max_length: i64,
}

impl QuerySegmentRule for QuerySegmentCodeLengthRule {
    fn check(&self, item: &QuerySegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuerySegmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuerySegmentRule for QuerySegmentDiscountRangeRule {
    fn check(&self, item: &QuerySegment) -> Vec<String> {
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
pub struct QuerySegmentMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuerySegmentRule for QuerySegmentMinimumRangeRule {
    fn check(&self, item: &QuerySegment) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query segment.
#[derive(Debug, Clone)]
pub struct QuerySegmentValidator {
    pub rules: Vec<Rc<dyn QuerySegmentRule>>,
}

impl QuerySegmentValidator {
    pub fn validate(&self, item: &QuerySegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuerySegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuerySegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_segment_validator() -> QuerySegmentValidator {
    QuerySegmentValidator {
        rules: vec![
            Rc::new(QuerySegmentCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn QuerySegmentRule>,
            Rc::new(QuerySegmentRequiredLabelRule {
                message: "query segment needs this field".to_string(),
            }),
            Rc::new(QuerySegmentCodeLengthRule { max_length: 1000 }),
            Rc::new(QuerySegmentDiscountRangeRule { min: 6.5, max: 2267.2 }),
            Rc::new(QuerySegmentMinimumRangeRule { min: 0.9, max: 3737.0 }),
        ],
    }
}
