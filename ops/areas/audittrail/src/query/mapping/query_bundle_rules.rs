use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query bundle must pass. Check returns its problems, or none.
pub trait QueryBundleRule: std::fmt::Debug {
    fn check(&self, item: &QueryBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBundleCodeLengthRule {
    pub max_length: i64,
}

impl QueryBundleRule for QueryBundleCodeLengthRule {
    fn check(&self, item: &QueryBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBundleCodePrefixRule {
    pub prefix: String,
}

impl QueryBundleRule for QueryBundleCodePrefixRule {
    fn check(&self, item: &QueryBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBundleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryBundleRule for QueryBundleStartsAtRangeRule {
    fn check(&self, item: &QueryBundle) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBundleCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryBundleRule for QueryBundleCostRangeRule {
    fn check(&self, item: &QueryBundle) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBundleRequiredCodeRule {
    pub message: String,
}

impl QueryBundleRule for QueryBundleRequiredCodeRule {
    fn check(&self, item: &QueryBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBundleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryBundleRule for QueryBundleRateRangeRule {
    fn check(&self, item: &QueryBundle) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query bundle.
#[derive(Debug, Clone)]
pub struct QueryBundleValidator {
    pub rules: Vec<Rc<dyn QueryBundleRule>>,
}

impl QueryBundleValidator {
    pub fn validate(&self, item: &QueryBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_bundle_validator() -> QueryBundleValidator {
    QueryBundleValidator {
        rules: vec![
            Rc::new(QueryBundleCodeLengthRule { max_length: 32 }) as Rc<dyn QueryBundleRule>,
            Rc::new(QueryBundleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(QueryBundleStartsAtRangeRule { min: 32, max: 255 }),
            Rc::new(QueryBundleCostRangeRule { min: 2.3, max: 3924.5 }),
            Rc::new(QueryBundleRequiredCodeRule {
                message: "query bundle needs a value".to_string(),
            }),
            Rc::new(QueryBundleRateRangeRule { min: 6.2, max: 3072.8 }),
        ],
    }
}
