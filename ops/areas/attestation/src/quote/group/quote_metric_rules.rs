use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote metric must pass. Check returns its problems, or none.
pub trait QuoteMetricRule: std::fmt::Debug {
    fn check(&self, item: &QuoteMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMetricRequiredTitleRule {
    pub message: String,
}

impl QuoteMetricRule for QuoteMetricRequiredTitleRule {
    fn check(&self, item: &QuoteMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMetricCodeLengthRule {
    pub max_length: i64,
}

impl QuoteMetricRule for QuoteMetricCodeLengthRule {
    fn check(&self, item: &QuoteMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMetricScopeLengthRule {
    pub max_length: i64,
}

impl QuoteMetricRule for QuoteMetricScopeLengthRule {
    fn check(&self, item: &QuoteMetric) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMetricRequiredScopeRule {
    pub message: String,
}

impl QuoteMetricRule for QuoteMetricRequiredScopeRule {
    fn check(&self, item: &QuoteMetric) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote metric.
#[derive(Debug, Clone)]
pub struct QuoteMetricValidator {
    pub rules: Vec<Rc<dyn QuoteMetricRule>>,
}

impl QuoteMetricValidator {
    pub fn validate(&self, item: &QuoteMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_metric_validator() -> QuoteMetricValidator {
    QuoteMetricValidator {
        rules: vec![
            Rc::new(QuoteMetricRequiredTitleRule {
                message: "quote metric needs an entry".to_string(),
            }) as Rc<dyn QuoteMetricRule>,
            Rc::new(QuoteMetricCodeLengthRule { max_length: 120 }),
            Rc::new(QuoteMetricScopeLengthRule { max_length: 80 }),
            Rc::new(QuoteMetricRequiredScopeRule {
                message: "quote metric needs a value".to_string(),
            }),
        ],
    }
}
