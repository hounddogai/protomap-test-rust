use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query exception must pass. Check returns its problems, or none.
pub trait QueryExceptionRule: std::fmt::Debug {
    fn check(&self, item: &QueryException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryExceptionAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl QueryExceptionRule for QueryExceptionAllowedContactEmailRule {
    fn check(&self, item: &QueryException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is not allowed", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryExceptionRequiredCodeRule {
    pub message: String,
}

impl QueryExceptionRule for QueryExceptionRequiredCodeRule {
    fn check(&self, item: &QueryException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryExceptionCodeLengthRule {
    pub max_length: i64,
}

impl QueryExceptionRule for QueryExceptionCodeLengthRule {
    fn check(&self, item: &QueryException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryExceptionCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QueryExceptionRule for QueryExceptionCostRangeRule {
    fn check(&self, item: &QueryException) -> Vec<String> {
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
pub struct QueryExceptionCodePrefixRule {
    pub prefix: String,
}

impl QueryExceptionRule for QueryExceptionCodePrefixRule {
    fn check(&self, item: &QueryException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryExceptionApprovedGuardRule {
    pub limit: f64,
}

impl QueryExceptionRule for QueryExceptionApprovedGuardRule {
    fn check(&self, item: &QueryException) -> Vec<String> {
        if item.approved && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while approved", item.cost, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query exception.
#[derive(Debug, Clone)]
pub struct QueryExceptionValidator {
    pub rules: Vec<Rc<dyn QueryExceptionRule>>,
}

impl QueryExceptionValidator {
    pub fn validate(&self, item: &QueryException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_exception_validator() -> QueryExceptionValidator {
    QueryExceptionValidator {
        rules: vec![
            Rc::new(QueryExceptionAllowedContactEmailRule {
                allowed: vec!["legacy".to_string(), "strict".to_string()],
            }) as Rc<dyn QueryExceptionRule>,
            Rc::new(QueryExceptionRequiredCodeRule {
                message: "query exception needs a value".to_string(),
            }),
            Rc::new(QueryExceptionCodeLengthRule { max_length: 86400 }),
            Rc::new(QueryExceptionCostRangeRule { min: 5.7, max: 2532.3 }),
            Rc::new(QueryExceptionCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(QueryExceptionApprovedGuardRule { limit: 5.4 }),
        ],
    }
}
