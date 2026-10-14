use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query mapping must pass. Check returns its problems, or none.
pub trait QueryMappingRule: std::fmt::Debug {
    fn check(&self, item: &QueryMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMappingRequiredSourceRule {
    pub message: String,
}

impl QueryMappingRule for QueryMappingRequiredSourceRule {
    fn check(&self, item: &QueryMapping) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMappingRequiredReviewerRule {
    pub message: String,
}

impl QueryMappingRule for QueryMappingRequiredReviewerRule {
    fn check(&self, item: &QueryMapping) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMappingRequiredNameRule {
    pub message: String,
}

impl QueryMappingRule for QueryMappingRequiredNameRule {
    fn check(&self, item: &QueryMapping) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query mapping.
#[derive(Debug, Clone)]
pub struct QueryMappingValidator {
    pub rules: Vec<Rc<dyn QueryMappingRule>>,
}

impl QueryMappingValidator {
    pub fn validate(&self, item: &QueryMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_mapping_validator() -> QueryMappingValidator {
    QueryMappingValidator {
        rules: vec![
            Rc::new(QueryMappingRequiredSourceRule {
                message: "query mapping needs a value".to_string(),
            }) as Rc<dyn QueryMappingRule>,
            Rc::new(QueryMappingRequiredReviewerRule {
                message: "query mapping needs a value".to_string(),
            }),
            Rc::new(QueryMappingRequiredNameRule {
                message: "query mapping needs a value".to_string(),
            }),
        ],
    }
}
