use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query variance must pass. Check returns its problems, or none.
pub trait QueryVarianceRule: std::fmt::Debug {
    fn check(&self, item: &QueryVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryVarianceCodePrefixRule {
    pub prefix: String,
}

impl QueryVarianceRule for QueryVarianceCodePrefixRule {
    fn check(&self, item: &QueryVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryVarianceVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryVarianceRule for QueryVarianceVersionRangeRule {
    fn check(&self, item: &QueryVariance) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryVarianceCodeLengthRule {
    pub max_length: i64,
}

impl QueryVarianceRule for QueryVarianceCodeLengthRule {
    fn check(&self, item: &QueryVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryVarianceRequiredLabelRule {
    pub message: String,
}

impl QueryVarianceRule for QueryVarianceRequiredLabelRule {
    fn check(&self, item: &QueryVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryVarianceRequiredSummaryRule {
    pub message: String,
}

impl QueryVarianceRule for QueryVarianceRequiredSummaryRule {
    fn check(&self, item: &QueryVariance) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryVarianceRequiredCodeRule {
    pub message: String,
}

impl QueryVarianceRule for QueryVarianceRequiredCodeRule {
    fn check(&self, item: &QueryVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query variance.
#[derive(Debug, Clone)]
pub struct QueryVarianceValidator {
    pub rules: Vec<Rc<dyn QueryVarianceRule>>,
}

impl QueryVarianceValidator {
    pub fn validate(&self, item: &QueryVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_variance_validator() -> QueryVarianceValidator {
    QueryVarianceValidator {
        rules: vec![
            Rc::new(QueryVarianceCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn QueryVarianceRule>,
            Rc::new(QueryVarianceVersionRangeRule { min: 64, max: 1000 }),
            Rc::new(QueryVarianceCodeLengthRule { max_length: 120 }),
            Rc::new(QueryVarianceRequiredLabelRule {
                message: "query variance needs an entry".to_string(),
            }),
            Rc::new(QueryVarianceRequiredSummaryRule {
                message: "query variance needs an entry".to_string(),
            }),
            Rc::new(QueryVarianceRequiredCodeRule {
                message: "query variance needs this field".to_string(),
            }),
        ],
    }
}
