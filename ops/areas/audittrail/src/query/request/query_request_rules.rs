use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query request must pass. Check returns its problems, or none.
pub trait QueryRequestRule: std::fmt::Debug {
    fn check(&self, item: &QueryRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryRequestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryRequestRule for QueryRequestCeilingRangeRule {
    fn check(&self, item: &QueryRequest) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryRequestRequiredReviewerRule {
    pub message: String,
}

impl QueryRequestRule for QueryRequestRequiredReviewerRule {
    fn check(&self, item: &QueryRequest) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryRequestRequiredCodeRule {
    pub message: String,
}

impl QueryRequestRule for QueryRequestRequiredCodeRule {
    fn check(&self, item: &QueryRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryRequestRequiredLabelRule {
    pub message: String,
}

impl QueryRequestRule for QueryRequestRequiredLabelRule {
    fn check(&self, item: &QueryRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query request.
#[derive(Debug, Clone)]
pub struct QueryRequestValidator {
    pub rules: Vec<Rc<dyn QueryRequestRule>>,
}

impl QueryRequestValidator {
    pub fn validate(&self, item: &QueryRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_request_validator() -> QueryRequestValidator {
    QueryRequestValidator {
        rules: vec![
            Rc::new(QueryRequestCeilingRangeRule { min: 7.6, max: 2735.1 }) as Rc<dyn QueryRequestRule>,
            Rc::new(QueryRequestRequiredReviewerRule {
                message: "query request needs this field".to_string(),
            }),
            Rc::new(QueryRequestRequiredCodeRule {
                message: "query request needs an entry".to_string(),
            }),
            Rc::new(QueryRequestRequiredLabelRule {
                message: "query request needs an entry".to_string(),
            }),
        ],
    }
}
